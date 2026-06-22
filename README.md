# 本实验的主要目的是实现进程及进程的管理。

# 

# 1\. 修改应用程序

# 

# （1）增加重要的系统调用

# 

# fork系统调用会创建一个新的进程；waitpid系统调用是的当前进程等待子进程结束，回收其资源并获得返回值；getpid系统调用获得当前进程的信息；exec系统调用将当前的进程地址空间清空并加载一个特定的可执行文件，然后返回用户态执行；read系统调用从文件中读取一段内容到缓冲区，主要目的是为了实现user shell。

# 

# 首先，修改user/src/syscall.rs增加上述系统调用。

# 

# //user/src/syscall.rs

# 

# const SYSCALL\_READ: usize = 63;

# const SYSCALL\_GETPID: usize = 172;

# const SYSCALL\_FORK: usize = 220;

# const SYSCALL\_EXEC: usize = 221;

# const SYSCALL\_WAITPID: usize = 260;

# 

# pub fn sys\_read(fd: usize, buffer: \&mut \[u8]) -> isize {

# &#x20;   syscall(SYSCALL\_READ, \[fd, buffer.as\_mut\_ptr() as usize, buffer.len()])

# }

# 

# pub fn sys\_getpid() -> isize {

# &#x20;   syscall(SYSCALL\_GETPID, \[0, 0, 0])

# }

# 

# pub fn sys\_fork() -> isize {

# &#x20;   syscall(SYSCALL\_FORK, \[0, 0, 0])

# }

# 

# pub fn sys\_exec(path: \&str) -> isize {

# &#x20;   syscall(SYSCALL\_EXEC, \[path.as\_ptr() as usize, 0, 0])

# }

# 

# pub fn sys\_waitpid(pid: isize, exit\_code: \*mut i32) -> isize {

# &#x20;   syscall(SYSCALL\_WAITPID, \[pid as usize, exit\_code as usize, 0])

# }

# 

# 接着，在user/src/lib.rs封装系统调用为应用程序使用的形式。

# 

# //user/src/lib.rs

# 

# pub fn read(fd: usize, buf: \&mut \[u8]) -> isize { sys\_read(fd, buf) }

# 

# pub fn getpid() -> isize { sys\_getpid() }

# pub fn fork() -> isize { sys\_fork() }

# pub fn exec(path: \&str) -> isize { sys\_exec(path) }

# pub fn wait(exit\_code: \&mut i32) -> isize {

# &#x20;   loop {

# &#x20;       match sys\_waitpid(-1, exit\_code as \*mut \_) {

# &#x20;           -2 => { yield\_(); }

# &#x20;           // -1 or a real pid

# &#x20;           exit\_pid => return exit\_pid,

# &#x20;       }

# &#x20;   }

# }

# 

# pub fn waitpid(pid: usize, exit\_code: \&mut i32) -> isize {

# &#x20;   loop {

# &#x20;       match sys\_waitpid(pid as isize, exit\_code as \*mut \_) {

# &#x20;           -2 => { yield\_(); }

# &#x20;           // -1 or a real pid

# &#x20;           exit\_pid => return exit\_pid,

# &#x20;       }

# &#x20;   }

# }

# 

# pub fn sleep(period\_ms: usize) {

# &#x20;   let start = sys\_get\_time();

# &#x20;   while sys\_get\_time() < start + period\_ms as isize {

# &#x20;       sys\_yield();

# &#x20;   }

# }

# 

# （2）实现用户初始程序initproc

# 

# //user/src/bin/initproc.rs

# 

# \#!\[no\_std]

# \#!\[no\_main]

# 

# \#\[macro\_use]

# extern crate user\_lib;

# 

# use user\_lib::{

# &#x20;   fork,

# &#x20;   wait,

# &#x20;   exec,

# &#x20;   yield\_,

# };

# 

# \#\[no\_mangle]

# fn main() -> i32 {

# &#x20;   if fork() == 0 {

# &#x20;       exec("user\_shell\\0");

# &#x20;   } else {

# &#x20;       loop {

# &#x20;           let mut exit\_code: i32 = 0;

# &#x20;           let pid = wait(\&mut exit\_code);

# &#x20;           if pid == -1 {

# &#x20;               yield\_();

# &#x20;               continue;

# &#x20;           }

# &#x20;           println!(

# &#x20;               "\[initproc] Released a zombie process, pid={}, exit\_code={}",

# &#x20;               pid,

# &#x20;               exit\_code,

# &#x20;           );

# &#x20;       }

# &#x20;   }

# &#x20;   0

# }

# 

# （3）实现shell程序

# 

# 首先基于sys\_read系统调用封装能够从标准输入读取一个字符的函数getchar。

# //user/src/console.rs 

# 

# use super::read;

# 

# const STDIN: usize = 0;

# 

# pub fn getchar() -> u8 {

# &#x20;   let mut c = \[0u8; 1];

# &#x20;   read(STDIN, \&mut c);

# &#x20;   c\[0]

# }

# 

# 然后，实现user shell程序。

# //user/src/bin/user\_shell.rs

# 

# \#!\[no\_std]

# \#!\[no\_main]

# 

# extern crate alloc;

# 

# \#\[macro\_use]

# extern crate user\_lib;

# 

# const LF: u8 = 0x0au8;

# const CR: u8 = 0x0du8;

# const DL: u8 = 0x7fu8;

# const BS: u8 = 0x08u8;

# 

# use alloc::string::String;

# use user\_lib::{fork, exec, waitpid};

# use user\_lib::console::getchar;

# 

# \#\[no\_mangle]

# pub fn main() -> i32 {

# &#x20;   println!("Rust user shell");

# &#x20;   let mut line: String = String::new();

# &#x20;   print!(">> ");

# &#x20;   loop {

# &#x20;       let c = getchar();

# &#x20;       match c {

# &#x20;           LF | CR => {

# &#x20;               println!("");

# &#x20;               if !line.is\_empty() {

# &#x20;                   line.push('\\0');

# &#x20;                   let pid = fork();

# &#x20;                   if pid == 0 {

# &#x20;                       // child process

# &#x20;                       if exec(line.as\_str()) == -1 {

# &#x20;                           println!("Error when executing!");

# &#x20;                           return -4;

# &#x20;                       }

# &#x20;                       unreachable!();

# &#x20;                   } else {

# &#x20;                       let mut exit\_code: i32 = 0;

# &#x20;                       let exit\_pid = waitpid(pid as usize, \&mut exit\_code);

# &#x20;                       assert\_eq!(pid, exit\_pid);

# &#x20;                       println!("Shell: Process {} exited with code {}", pid, exit\_code);

# &#x20;                   }

# &#x20;                   line.clear();

# &#x20;               }

# &#x20;               print!(">> ");

# &#x20;           }

# &#x20;           BS | DL => {

# &#x20;               if !line.is\_empty() {

# &#x20;                   print!("{}", BS as char);

# &#x20;                   print!(" ");

# &#x20;                   print!("{}", BS as char);

# &#x20;                   line.pop();

# &#x20;               }

# &#x20;           }

# &#x20;           \_ => {

# &#x20;               print!("{}", c as char);

# &#x20;               line.push(c as char);

# &#x20;           }

# &#x20;       }

# &#x20;   }

# }

# 

# 因为Rust的可边长字符串类型String基于动态内存分配，因此还需要在用户库user\_lib中支持动态内存分配。

# 

# //usr/src/lib.rs

# \#!\[feature(alloc\_error\_handler)]

# 

# use buddy\_system\_allocator::LockedHeap;

# 

# const USER\_HEAP\_SIZE: usize = 16384;

# static mut HEAP\_SPACE: \[u8; USER\_HEAP\_SIZE] = \[0; USER\_HEAP\_SIZE];

# 

# \#\[global\_allocator]

# static HEAP: LockedHeap = LockedHeap::empty();

# 

# \#\[alloc\_error\_handler]

# pub fn handle\_alloc\_error(layout: core::alloc::Layout) -> ! {

# &#x20;   panic!("Heap allocation error, layout = {:?}", layout);

# }

# 

# \#\[no\_mangle]

# \#\[link\_section = ".text.entry"]

# pub extern "C" fn \_start() -> ! {

# &#x20;   unsafe {

# &#x20;       HEAP.lock()

# &#x20;           .init(HEAP\_SPACE.as\_ptr() as usize, USER\_HEAP\_SIZE);

# &#x20;   }

# &#x20;   exit(main());

# }

# 

# 注意需要修改user下的Cargo.toml配置文件，增加buddy\_system\_allocator的依赖。具体增加如下内容：

# \[dependencies]

# buddy\_system\_allocator = "0.6"

# 

# 另外，其他应用程序的实现就不再这里一一列出，请参考示例代码。

# 

# 

# 2\. 在内核中增加系统调用

# 

# 首先，修改os/src/syscall/mod.rs增加fork、waitpid、getpid、read系统调用。

# 

# //os/src/syscall/mod.rs

# 

# const SYSCALL\_READ: usize = 63;

# const SYSCALL\_GETPID: usize = 172;

# const SYSCALL\_FORK: usize = 220;

# const SYSCALL\_EXEC: usize = 221;

# const SYSCALL\_WAITPID: usize = 260;

# 

# mod fs;

# mod process;

# 

# use fs::\*;

# use process::\*;

# 

# pub fn syscall(syscall\_id: usize, args: \[usize; 3]) -> isize {

# &#x20;   match syscall\_id {

# &#x20;       SYSCALL\_READ => sys\_read(args\[0], args\[1] as \*const u8, args\[2]),

# &#x20;       SYSCALL\_WRITE => sys\_write(args\[0], args\[1] as \*const u8, args\[2]),

# &#x20;       SYSCALL\_EXIT => sys\_exit(args\[0] as i32),

# &#x20;       SYSCALL\_YIELD => sys\_yield(),

# &#x20;       SYSCALL\_GET\_TIME => sys\_get\_time(),

# &#x20;       SYSCALL\_GETPID => sys\_getpid(),

# &#x20;       SYSCALL\_FORK => sys\_fork(),

# &#x20;       SYSCALL\_EXEC => sys\_exec(args\[0] as \*const u8),

# &#x20;       SYSCALL\_WAITPID => sys\_waitpid(args\[0] as isize, args\[1] as \*mut i32),

# &#x20;       \_ => panic!("Unsupported syscall\_id: {}", syscall\_id),

# &#x20;   }

# }

# 

# 

# 然后，修改os/src/syscall/fs.rs，实现sys\_read系统调用。

# 

# //os/src/syscall/fs.rs

# 

# use crate::task::{current\_user\_token, suspend\_current\_and\_run\_next};

# use crate::sbi::console\_getchar;

# const FD\_STDIN: usize = 0;

# 

# 

# pub fn sys\_read(fd: usize, buf: \*const u8, len: usize) -> isize {

# &#x20;   match fd {

# &#x20;       FD\_STDIN => {

# &#x20;           assert\_eq!(len, 1, "Only support len = 1 in sys\_read!");

# &#x20;           let mut c: usize;

# &#x20;           loop {

# &#x20;               c = console\_getchar();

# &#x20;               if c == 0 {

# &#x20;                   suspend\_current\_and\_run\_next();

# &#x20;                   continue;

# &#x20;               } else {

# &#x20;                   break;

# &#x20;               }

# &#x20;           }

# &#x20;           let ch = c as u8;

# &#x20;           let mut buffers = translated\_byte\_buffer(current\_user\_token(), buf, len);

# &#x20;           unsafe { buffers\[0].as\_mut\_ptr().write\_volatile(ch); }

# &#x20;           1

# &#x20;       }

# &#x20;       \_ => {

# &#x20;           panic!("Unsupported fd in sys\_read!");

# &#x20;       }

# &#x20;   }

# }

# 

# 其中，suspend\_current\_and\_run\_next函数是暂停当前的任务并切换到下一个任务，具体实现将在后面介绍。

# 

# 然后，修改os/src/syscall/process.rs实现其他系统调用。

# //os/src/syscall/process.rs

# 

# use crate::task::{

# &#x20;   suspend\_current\_and\_run\_next,

# &#x20;   exit\_current\_and\_run\_next,

# &#x20;   current\_task,

# &#x20;   current\_user\_token,

# &#x20;   add\_task,

# };

# 

# use crate::mm::{

# &#x20;   translated\_str,

# &#x20;   translated\_refmut,

# };

# use crate::loader::get\_app\_data\_by\_name;

# use alloc::sync::Arc;

# 

# pub fn sys\_getpid() -> isize {

# &#x20;   current\_task().unwrap().pid.0 as isize

# }

# 

# pub fn sys\_fork() -> isize {

# &#x20;   let current\_task = current\_task().unwrap();

# &#x20;   let new\_task = current\_task.fork();

# &#x20;   let new\_pid = new\_task.pid.0;

# &#x20;   // modify trap context of new\_task, because it returns immediately after switching

# &#x20;   let trap\_cx = new\_task.inner\_exclusive\_access().get\_trap\_cx();

# &#x20;   // we do not have to move to next instruction since we have done it before

# &#x20;   // for child process, fork returns 0

# &#x20;   trap\_cx.x\[10] = 0;

# &#x20;   // add new task to scheduler

# &#x20;   add\_task(new\_task);

# &#x20;   new\_pid as isize

# }

# 

# pub fn sys\_exec(path: \*const u8) -> isize {

# &#x20;   let token = current\_user\_token();

# &#x20;   let path = translated\_str(token, path);

# &#x20;   if let Some(data) = get\_app\_data\_by\_name(path.as\_str()) {

# &#x20;       let task = current\_task().unwrap();

# &#x20;       task.exec(data);

# &#x20;       0

# &#x20;   } else {

# &#x20;       -1

# &#x20;   }

# }

# 

# /// If there is not a child process whose pid is same as given, return -1.

# /// Else if there is a child process but it is still running, return -2.

# pub fn sys\_waitpid(pid: isize, exit\_code\_ptr: \*mut i32) -> isize {

# &#x20;   let task = current\_task().unwrap();

# &#x20;   // find a child process

# 

# &#x20;   // ---- access current TCB exclusively

# &#x20;   let mut inner = task.inner\_exclusive\_access();

# &#x20;   if inner.children

# &#x20;       .iter()

# &#x20;       .find(|p| {pid == -1 || pid as usize == p.getpid()})

# &#x20;       .is\_none() {

# &#x20;       return -1;

# &#x20;       // ---- release current PCB

# &#x20;   }

# &#x20;   let pair = inner.children

# &#x20;       .iter()

# &#x20;       .enumerate()

# &#x20;       .find(|(\_, p)| {

# &#x20;           // ++++ temporarily access child PCB lock exclusively

# &#x20;           p.inner\_exclusive\_access().is\_zombie() \&\& (pid == -1 || pid as usize == p.getpid())

# &#x20;           // ++++ release child PCB

# &#x20;       });

# &#x20;   if let Some((idx, \_)) = pair {

# &#x20;       let child = inner.children.remove(idx);

# &#x20;       // confirm that child will be deallocated after removing from children list

# &#x20;       assert\_eq!(Arc::strong\_count(\&child), 1);

# &#x20;       let found\_pid = child.getpid();

# &#x20;       // ++++ temporarily access child TCB exclusively

# &#x20;       let exit\_code = child.inner\_exclusive\_access().exit\_code;

# &#x20;       // ++++ release child PCB

# &#x20;       \*translated\_refmut(inner.memory\_set.token(), exit\_code\_ptr) = exit\_code;

# &#x20;       found\_pid as isize

# &#x20;   } else {

# &#x20;       -2

# &#x20;   }

# &#x20;   // ---- release current PCB lock automatically

# }

# 

# 

# 

# 3\. 应用的链接与加载

# 

# （1）基于名字的应用链接

# 

# 因为实现exec系统调用需要根据应用程序的名字获取ELF格式的数据，因此需要修改链接和加载接口。

# 

# 修改编译链接辅助文件os/build.rs。

# //os/build.rs

# 

# writeln!(f, r#"

# &#x20;   .global \_app\_names

# \_app\_names:"#)?;

# &#x20;   for app in apps.iter() {

# &#x20;       writeln!(f, r#"    .string "{}""#, app)?;

# &#x20;   }

# 

# 

# （2）基于名字的应用加载

# 

# 应用加载子模块loader.rs会用一个全局可见的只读向量APP\_NAMES按照顺序吧所有应用的名字保存在内存中。

# 

# //os/src/loader.rs

# 

# use alloc::vec::Vec;

# use lazy\_static::\*;

# 

# lazy\_static! {

# &#x20;   static ref APP\_NAMES: Vec<\&'static str> = {

# &#x20;       let num\_app = get\_num\_app();

# &#x20;       extern "C" { fn \_app\_names(); }

# &#x20;       let mut start = \_app\_names as usize as \*const u8;

# &#x20;       let mut v = Vec::new();

# &#x20;       unsafe {

# &#x20;           for \_ in 0..num\_app {

# &#x20;               let mut end = start;

# &#x20;               while end.read\_volatile() != '\\0' as u8 {

# &#x20;                   end = end.add(1);

# &#x20;               }

# &#x20;               let slice = core::slice::from\_raw\_parts(start, end as usize - start as usize);

# &#x20;               let str = core::str::from\_utf8(slice).unwrap();

# &#x20;               v.push(str);

# &#x20;               start = end.add(1);

# &#x20;           }

# &#x20;       }

# &#x20;       v

# &#x20;   };

# }

# 

# \#\[allow(unused)]

# pub fn get\_app\_data\_by\_name(name: \&str) -> Option<\&'static \[u8]> {

# &#x20;   let num\_app = get\_num\_app();

# &#x20;   (0..num\_app)

# &#x20;       .find(|\&i| APP\_NAMES\[i] == name)

# &#x20;       .map(|i| get\_app\_data(i))

# }

# 

# pub fn list\_apps() {

# &#x20;   println!("/\*\*\*\* APPS \*\*\*\*");

# &#x20;   for app in APP\_NAMES.iter() {

# &#x20;       println!("{}", app);

# &#x20;   }

# &#x20;   println!("\*\*\*\*\*\*\*\*\*\*\*\*\*\*/");

# }

# 

# 

# 

# 4\. 进程标识符与内核栈

# 

# （1）实现进程标识符

# 进程标识应当是唯一的，我们将其抽象为一个PidHandle类型。

# 

# //os/src/task/pid.rs # 注意增加如下代码：

# 

# pub struct PidHandle(pub usize);

# 

# 类似于之前的物理页帧的管理，我们实现一个进程标识符分配器PID\_ALLOCATOR。

# 

# //os/src/task/pid.rs

# 

# struct PidAllocator {

# &#x20;   current: usize,

# &#x20;   recycled: Vec<usize>,

# }

# 

# impl PidAllocator {

# &#x20;   pub fn new() -> Self {

# &#x20;       PidAllocator {

# &#x20;           current: 0,

# &#x20;           recycled: Vec::new(),

# &#x20;       }

# &#x20;   }

# &#x20;   pub fn alloc(\&mut self) -> PidHandle {

# &#x20;       if let Some(pid) = self.recycled.pop() {

# &#x20;           PidHandle(pid)

# &#x20;       } else {

# &#x20;           self.current += 1;

# &#x20;           PidHandle(self.current - 1)

# &#x20;       }

# &#x20;   }

# &#x20;   pub fn dealloc(\&mut self, pid: usize) {

# &#x20;       assert!(pid < self.current);

# &#x20;       assert!(

# &#x20;           self.recycled.iter().find(|ppid| \*\*ppid == pid).is\_none(),

# &#x20;           "pid {} has been deallocated!", pid

# &#x20;       );

# &#x20;       self.recycled.push(pid);

# &#x20;   }

# }

# 

# lazy\_static! {

# &#x20;   static ref PID\_ALLOCATOR : UPSafeCell<PidAllocator> = unsafe {

# &#x20;       UPSafeCell::new(PidAllocator::new())

# &#x20;   };

# }

# 

# 我们还需要封装一个全局的进程标识分配接口pid\_alloc。

# //os/src/task/pid.rs

# 

# pub fn pid\_alloc() -> PidHandle {

# &#x20;   PID\_ALLOCATOR.exclusive\_access().alloc()

# }

# 

# 同时，为了允许资源的自动回收，还需要为PidHandle实现Drop Trait。

# 

# //os/src/task/pid.rs

# 

# impl Drop for PidHandle {

# &#x20;   fn drop(\&mut self) {

# &#x20;       //println!("drop pid {}", self.0);

# &#x20;       PID\_ALLOCATOR.exclusive\_access().dealloc(self.0);

# &#x20;   }

# }

# 

# （2）在内核栈中保存进程标识符

# 

# 重新定义内核栈。

# 

# //os/src/task/pid.rs

# 

# use alloc::vec::Vec;

# use lazy\_static::\*;

# use crate::sync::UPSafeCell;

# use crate::mm::{KERNEL\_SPACE, MapPermission, VirtAddr};

# use crate::config::{

# &#x20;   PAGE\_SIZE,

# &#x20;   TRAMPOLINE,

# &#x20;   KERNEL\_STACK\_SIZE,

# };

# 

# 

# pub struct KernelStack {

# &#x20;   pid: usize,

# }

# 

# 

# 实现如下方法。

# 

# /// Return (bottom, top) of a kernel stack in kernel space.

# pub fn kernel\_stack\_position(app\_id: usize) -> (usize, usize) {

# &#x20;   let top = TRAMPOLINE - app\_id \* (KERNEL\_STACK\_SIZE + PAGE\_SIZE);

# &#x20;   let bottom = top - KERNEL\_STACK\_SIZE;

# &#x20;   (bottom, top)

# }

# 

# impl KernelStack {

# &#x20;   pub fn new(pid\_handle: \&PidHandle) -> Self {

# &#x20;       let pid = pid\_handle.0;

# &#x20;       let (kernel\_stack\_bottom, kernel\_stack\_top) = kernel\_stack\_position(pid);

# &#x20;       KERNEL\_SPACE

# &#x20;           .exclusive\_access()

# &#x20;           .insert\_framed\_area(

# &#x20;               kernel\_stack\_bottom.into(),

# &#x20;               kernel\_stack\_top.into(),

# &#x20;               MapPermission::R | MapPermission::W,

# &#x20;           );

# &#x20;       KernelStack {

# &#x20;           pid: pid\_handle.0,

# &#x20;       }

# &#x20;   }

# &#x20;   #\[allow(unused)]

# &#x20;   pub fn push\_on\_top<T>(\&self, value: T) -> \*mut T where

# &#x20;       T: Sized, {

# &#x20;       let kernel\_stack\_top = self.get\_top();

# &#x20;       let ptr\_mut = (kernel\_stack\_top - core::mem::size\_of::<T>()) as \*mut T;

# &#x20;       unsafe { \*ptr\_mut = value; }

# &#x20;       ptr\_mut

# &#x20;   }

# &#x20;   pub fn get\_top(\&self) -> usize {

# &#x20;       let (\_, kernel\_stack\_top) = kernel\_stack\_position(self.pid);

# &#x20;       kernel\_stack\_top

# &#x20;   }

# }

# 

# 同时也需要实现KernelStack 的Drop Trait以便KernelStack生命周期结束时回收相应的物理页帧。

# 

# impl Drop for KernelStack {

# &#x20;   fn drop(\&mut self) {

# &#x20;       let (kernel\_stack\_bottom, \_) = kernel\_stack\_position(self.pid);

# &#x20;       let kernel\_stack\_bottom\_va: VirtAddr = kernel\_stack\_bottom.into();

# &#x20;       KERNEL\_SPACE

# &#x20;           .exclusive\_access()

# &#x20;           .remove\_area\_with\_start\_vpn(kernel\_stack\_bottom\_va.into());

# &#x20;   }

# }

# 

# 相应的，还需要修改os/src/mm/memory\_set.rs

# 

# impl MemorySet {

# &#x20;   pub fn remove\_area\_with\_start\_vpn(\&mut self, start\_vpn: VirtPageNum) {

# &#x20;       if let Some((idx, area)) = self.areas.iter\_mut().enumerate()

# &#x20;           .find(|(\_, area)| area.vpn\_range.get\_start() == start\_vpn) {

# &#x20;           area.unmap(\&mut self.page\_table);

# &#x20;           self.areas.remove(idx);

# &#x20;       }

# &#x20;   }

# }

# 

# 

# 5\. 修改实现进程控制块

# 

# 修改原本的TaskControlBlock实现进程控制块的功能。

# 

# //os/src/task/task.rs

# 

# use crate::sync::UPSafeCell;

# use core::cell::RefMut;

# use super::{PidHandle, pid\_alloc, KernelStack};

# use alloc::sync::{Weak, Arc};

# use alloc::vec::Vec;

# 

# pub struct TaskControlBlock {

# &#x20;   // immutable

# &#x20;   pub pid: PidHandle,

# &#x20;   pub kernel\_stack: KernelStack,

# &#x20;   // mutable

# &#x20;   inner: UPSafeCell<TaskControlBlockInner>,

# }

# 

# pub struct TaskControlBlockInner {

# &#x20;   pub trap\_cx\_ppn: PhysPageNum,

# &#x20;   pub base\_size: usize,

# &#x20;   pub task\_cx: TaskContext,

# &#x20;   pub task\_status: TaskStatus,

# &#x20;   pub memory\_set: MemorySet,

# &#x20;   pub parent: Option<Weak<TaskControlBlock>>,

# &#x20;   pub children: Vec<Arc<TaskControlBlock>>,

# &#x20;   pub exit\_code: i32,

# }

# 

# TaskControlBlockInner实现以下方法：

# 

# impl TaskControlBlockInner {

# &#x20;   /\*

# &#x20;   pub fn get\_task\_cx\_ptr2(\&self) -> \*const usize {

# &#x20;       \&self.task\_cx\_ptr as \*const usize

# &#x20;   }

# &#x20;   \*/

# &#x20;   pub fn get\_trap\_cx(\&self) -> \&'static mut TrapContext {

# &#x20;       self.trap\_cx\_ppn.get\_mut()

# &#x20;   }

# &#x20;   pub fn get\_user\_token(\&self) -> usize {

# &#x20;       self.memory\_set.token()

# &#x20;   }

# &#x20;   fn get\_status(\&self) -> TaskStatus {

# &#x20;       self.task\_status

# &#x20;   }

# &#x20;   pub fn is\_zombie(\&self) -> bool {

# &#x20;       self.get\_status() == TaskStatus::Zombie

# &#x20;   }

# }

# 

# TaskControlBlock实现以下方法：

# 

# impl TaskControlBlock {

# &#x20;   pub fn inner\_exclusive\_access(\&self) -> RefMut<'\_, TaskControlBlockInner> {

# &#x20;       self.inner.exclusive\_access()

# &#x20;   }

# &#x20;   pub fn new(elf\_data: \&\[u8]) -> Self {

# &#x20;       // memory\_set with elf program headers/trampoline/trap context/user stack

# &#x20;       let (memory\_set, user\_sp, entry\_point) = MemorySet::from\_elf(elf\_data);

# &#x20;       let trap\_cx\_ppn = memory\_set

# &#x20;           .translate(VirtAddr::from(TRAP\_CONTEXT).into())

# &#x20;           .unwrap()

# &#x20;           .ppn();

# &#x20;       // alloc a pid and a kernel stack in kernel space

# &#x20;       let pid\_handle = pid\_alloc();

# &#x20;       let kernel\_stack = KernelStack::new(\&pid\_handle);

# &#x20;       let kernel\_stack\_top = kernel\_stack.get\_top();

# &#x20;       // push a task context which goes to trap\_return to the top of kernel stack

# &#x20;       let task\_control\_block = Self {

# &#x20;           pid: pid\_handle,

# &#x20;           kernel\_stack,

# &#x20;           inner: unsafe { UPSafeCell::new(TaskControlBlockInner {

# &#x20;               trap\_cx\_ppn,

# &#x20;               base\_size: user\_sp,

# &#x20;               task\_cx: TaskContext::goto\_trap\_return(kernel\_stack\_top),

# &#x20;               task\_status: TaskStatus::Ready,

# &#x20;               memory\_set,

# &#x20;               parent: None,

# &#x20;               children: Vec::new(),

# &#x20;               exit\_code: 0,

# &#x20;           })},

# &#x20;       };

# &#x20;       // prepare TrapContext in user space

# &#x20;       let trap\_cx = task\_control\_block.inner\_exclusive\_access().get\_trap\_cx();

# &#x20;       \*trap\_cx = TrapContext::app\_init\_context(

# &#x20;           entry\_point,

# &#x20;           user\_sp,

# &#x20;           KERNEL\_SPACE.exclusive\_access().token(),

# &#x20;           kernel\_stack\_top,

# &#x20;           trap\_handler as usize,

# &#x20;       );

# &#x20;       task\_control\_block

# &#x20;   }

# &#x20;   pub fn exec(\&self, elf\_data: \&\[u8]) {

# &#x20;       // memory\_set with elf program headers/trampoline/trap context/user stack

# &#x20;       let (memory\_set, user\_sp, entry\_point) = MemorySet::from\_elf(elf\_data);

# &#x20;       let trap\_cx\_ppn = memory\_set

# &#x20;           .translate(VirtAddr::from(TRAP\_CONTEXT).into())

# &#x20;           .unwrap()

# &#x20;           .ppn();

# 

# &#x20;       // \*\*\*\* access inner exclusively

# &#x20;       let mut inner = self.inner\_exclusive\_access();

# &#x20;       // substitute memory\_set

# &#x20;       inner.memory\_set = memory\_set;

# &#x20;       // update trap\_cx ppn

# &#x20;       inner.trap\_cx\_ppn = trap\_cx\_ppn;

# &#x20;       // initialize trap\_cx

# &#x20;       let trap\_cx = inner.get\_trap\_cx();

# &#x20;       \*trap\_cx = TrapContext::app\_init\_context(

# &#x20;           entry\_point,

# &#x20;           user\_sp,

# &#x20;           KERNEL\_SPACE.exclusive\_access().token(),

# &#x20;           self.kernel\_stack.get\_top(),

# &#x20;           trap\_handler as usize,

# &#x20;       );

# &#x20;       // \*\*\*\* release inner automatically

# &#x20;   }

# &#x20;   pub fn fork(self: \&Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {

# &#x20;       // ---- access parent PCB exclusively

# &#x20;       let mut parent\_inner = self.inner\_exclusive\_access();

# &#x20;       // copy user space(include trap context)

# &#x20;       let memory\_set = MemorySet::from\_existed\_user(

# &#x20;           \&parent\_inner.memory\_set

# &#x20;       );

# &#x20;       let trap\_cx\_ppn = memory\_set

# &#x20;           .translate(VirtAddr::from(TRAP\_CONTEXT).into())

# &#x20;           .unwrap()

# &#x20;           .ppn();

# &#x20;       // alloc a pid and a kernel stack in kernel space

# &#x20;       let pid\_handle = pid\_alloc();

# &#x20;       let kernel\_stack = KernelStack::new(\&pid\_handle);

# &#x20;       let kernel\_stack\_top = kernel\_stack.get\_top();

# &#x20;       let task\_control\_block = Arc::new(TaskControlBlock {

# &#x20;           pid: pid\_handle,

# &#x20;           kernel\_stack,

# &#x20;           inner: unsafe { UPSafeCell::new(TaskControlBlockInner {

# &#x20;               trap\_cx\_ppn,

# &#x20;               base\_size: parent\_inner.base\_size,

# &#x20;               task\_cx: TaskContext::goto\_trap\_return(kernel\_stack\_top),

# &#x20;               task\_status: TaskStatus::Ready,

# &#x20;               memory\_set,

# &#x20;               parent: Some(Arc::downgrade(self)),

# &#x20;               children: Vec::new(),

# &#x20;               exit\_code: 0,

# &#x20;           })},

# &#x20;       });

# &#x20;       // add child

# &#x20;       parent\_inner.children.push(task\_control\_block.clone());

# &#x20;       // modify kernel\_sp in trap\_cx

# &#x20;       // \*\*\*\* access children PCB exclusively

# &#x20;       let trap\_cx = task\_control\_block.inner\_exclusive\_access().get\_trap\_cx();

# &#x20;       trap\_cx.kernel\_sp = kernel\_stack\_top;

# &#x20;       // return

# &#x20;       task\_control\_block

# &#x20;       // ---- release parent PCB automatically

# &#x20;       // \*\*\*\* release children PCB automatically

# &#x20;   }

# &#x20;   pub fn getpid(\&self) -> usize {

# &#x20;       self.pid.0

# &#x20;   }

# }

# 

# 同时修改TaskStatus的状态。

# \#\[derive(Copy, Clone, PartialEq)]

# pub enum TaskStatus {

# &#x20;   Ready,

# &#x20;   Running,

# &#x20;   Zombie,

# }

# 

# 

# 6\. 实现任务管理器

# 

# 修改任务管理器，将部分任务管理功能移到处理器管理中。

# 

# //os/src/task/manager.rs

# 

# use crate::sync::UPSafeCell;

# use super::TaskControlBlock;

# use alloc::collections::VecDeque;

# use alloc::sync::Arc;

# use lazy\_static::\*;

# 

# pub struct TaskManager {

# &#x20;   ready\_queue: VecDeque<Arc<TaskControlBlock>>,

# }

# 

# /// A simple FIFO scheduler.

# impl TaskManager {

# &#x20;   pub fn new() -> Self {

# &#x20;       Self { ready\_queue: VecDeque::new(), }

# &#x20;   }

# &#x20;   pub fn add(\&mut self, task: Arc<TaskControlBlock>) {

# &#x20;       self.ready\_queue.push\_back(task);

# &#x20;   }

# &#x20;   pub fn fetch(\&mut self) -> Option<Arc<TaskControlBlock>> {

# &#x20;       self.ready\_queue.pop\_front()

# &#x20;   }

# }

# 

# lazy\_static! {

# &#x20;   pub static ref TASK\_MANAGER: UPSafeCell<TaskManager> = unsafe {

# &#x20;       UPSafeCell::new(TaskManager::new())

# &#x20;   };

# }

# 

# pub fn add\_task(task: Arc<TaskControlBlock>) {

# &#x20;   TASK\_MANAGER.exclusive\_access().add(task);

# }

# 

# pub fn fetch\_task() -> Option<Arc<TaskControlBlock>> {

# &#x20;   TASK\_MANAGER.exclusive\_access().fetch()

# }

# 

# 

# 7\. 增加处理器管理结构

# 

# 实现处理器管理结构Processor，完成从任务管理器分离的维护CPU状态的部分功能。

# 

# //os/src/task/processor.rs

# 

# use super::{TaskContext, TaskControlBlock};

# use alloc::sync::Arc;

# use lazy\_static::\*;

# use super::{fetch\_task, TaskStatus};

# use super::\_\_switch;

# use crate::trap::TrapContext;

# use crate::sync::UPSafeCell;

# 

# pub struct Processor {

# &#x20;   current: Option<Arc<TaskControlBlock>>,

# &#x20;   idle\_task\_cx: TaskContext,

# }

# 

# impl Processor {

# &#x20;   pub fn new() -> Self {

# &#x20;       Self {

# &#x20;           current: None,

# &#x20;           idle\_task\_cx: TaskContext::zero\_init(),

# &#x20;       }

# &#x20;   }

# &#x20;   fn get\_idle\_task\_cx\_ptr(\&mut self) -> \*mut TaskContext {

# &#x20;       \&mut self.idle\_task\_cx as \*mut \_

# &#x20;   }

# &#x20;   pub fn take\_current(\&mut self) -> Option<Arc<TaskControlBlock>> {

# &#x20;       self.current.take()

# &#x20;   }

# &#x20;   pub fn current(\&self) -> Option<Arc<TaskControlBlock>> {

# &#x20;       self.current.as\_ref().map(|task| Arc::clone(task))

# &#x20;   }

# }

# 

# lazy\_static! {

# &#x20;   pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe {

# &#x20;       UPSafeCell::new(Processor::new())

# &#x20;   };

# }

# 

# pub fn run\_tasks() {

# &#x20;   loop {

# &#x20;       let mut processor = PROCESSOR.exclusive\_access();

# &#x20;       if let Some(task) = fetch\_task() {

# &#x20;           let idle\_task\_cx\_ptr = processor.get\_idle\_task\_cx\_ptr();

# &#x20;           // access coming task TCB exclusively

# &#x20;           let mut task\_inner = task.inner\_exclusive\_access();

# &#x20;           let next\_task\_cx\_ptr = \&task\_inner.task\_cx as \*const TaskContext;

# &#x20;           task\_inner.task\_status = TaskStatus::Running;

# &#x20;           drop(task\_inner);

# &#x20;           // release coming task TCB manually

# &#x20;           processor.current = Some(task);

# &#x20;           // release processor manually

# &#x20;           drop(processor);

# &#x20;           unsafe {

# &#x20;               \_\_switch(

# &#x20;                   idle\_task\_cx\_ptr,

# &#x20;                   next\_task\_cx\_ptr,

# &#x20;               );

# &#x20;           }

# &#x20;       }

# &#x20;   }

# }

# 

# pub fn take\_current\_task() -> Option<Arc<TaskControlBlock>> {

# &#x20;   PROCESSOR.exclusive\_access().take\_current()

# }

# 

# pub fn current\_task() -> Option<Arc<TaskControlBlock>> {

# &#x20;   PROCESSOR.exclusive\_access().current()

# }

# 

# pub fn current\_user\_token() -> usize {

# &#x20;   let task = current\_task().unwrap();

# &#x20;   let token = task.inner\_exclusive\_access().get\_user\_token();

# &#x20;   token

# }

# 

# pub fn current\_trap\_cx() -> \&'static mut TrapContext {

# &#x20;   current\_task().unwrap().inner\_exclusive\_access().get\_trap\_cx()

# }

# 

# pub fn schedule(switched\_task\_cx\_ptr: \*mut TaskContext) {

# &#x20;   let mut processor = PROCESSOR.exclusive\_access();

# &#x20;   let idle\_task\_cx\_ptr = processor.get\_idle\_task\_cx\_ptr();

# &#x20;   drop(processor);

# &#x20;   unsafe {

# &#x20;       \_\_switch(

# &#x20;           switched\_task\_cx\_ptr,

# &#x20;           idle\_task\_cx\_ptr,

# &#x20;       );

# &#x20;   }

# }

# 

# 

# 8\. 创建初始进程

# 

# 内核初始化完成后，将会调用task子模块的add\_initproc将初始进程initproc加入任务管理器。在这之前要初始化初始进程的进程控制块。

# 

# 首先删除os/src/task/mod.rs中TaskManager和TaskManagerInner相关的实现。然后增加如下代码：

# 

# //os/src/task/mod.rs

# 

# use crate::loader::get\_app\_data\_by\_name;

# use manager::add\_task;

# 

# lazy\_static! {

# &#x20;   pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(

# &#x20;       TaskControlBlock::new(get\_app\_data\_by\_name("initproc").unwrap())

# &#x20;   );

# }

# 

# pub fn add\_initproc() {

# &#x20;   add\_task(INITPROC.clone());

# }

# 

# 

# 9\. 进程调度机制

# 

# 通过调用 task 子模块提供的 suspend\_current\_and\_run\_next 函数可以暂停当前任务并切换到另外一个任务。因为进程概念的引入，其实现需要更改。

# 

# //os/src/task/mod.rs

# 

# pub fn suspend\_current\_and\_run\_next() {

# &#x20;   // There must be an application running.

# &#x20;   let task = take\_current\_task().unwrap();

# 

# &#x20;   // ---- access current TCB exclusively

# &#x20;   let mut task\_inner = task.inner\_exclusive\_access();

# &#x20;   let task\_cx\_ptr = \&mut task\_inner.task\_cx as \*mut TaskContext;

# &#x20;   // Change status to Ready

# &#x20;   task\_inner.task\_status = TaskStatus::Ready;

# &#x20;   drop(task\_inner);

# &#x20;   // ---- release current PCB

# 

# &#x20;   // push back to ready queue.

# &#x20;   add\_task(task);

# &#x20;   // jump to scheduling cycle

# &#x20;   schedule(task\_cx\_ptr);

# }

# 

# 

# 10\. 进程的生成机制

# 

# 在内核中只有初始进程initproc是手动生成的，其他的进程由初始进程直接或间接fork出来，然后再调用exec系统调用加载并执行可执行文件。所以，进程的生成机制由fork和exec两个系统调用来完成。

# 

# 实现fork系统调用最关键的是为子进程创建一个和父进程几乎相同的地址空间。具体实现如下。

# 

# //os/src/mm/memory\_set.rs

# 

# impl MapArea {

# &#x20;   pub fn from\_another(another: \&MapArea) -> Self {

# &#x20;       Self {

# &#x20;           vpn\_range: VPNRange::new(another.vpn\_range.get\_start(), another.vpn\_range.get\_end()),

# &#x20;           data\_frames: BTreeMap::new(),

# &#x20;           map\_type: another.map\_type,

# &#x20;           map\_perm: another.map\_perm,

# &#x20;       }

# &#x20;   }

# }

# 

# impl MemorySet {

# &#x20;   pub fn from\_existed\_user(user\_space: \&MemorySet) -> MemorySet {

# &#x20;       let mut memory\_set = Self::new\_bare();

# &#x20;       // map trampoline

# &#x20;       memory\_set.map\_trampoline();

# &#x20;       // copy data sections/trap\_context/user\_stack

# &#x20;       for area in user\_space.areas.iter() {

# &#x20;           let new\_area = MapArea::from\_another(area);

# &#x20;           memory\_set.push(new\_area, None);

# &#x20;           // copy data from another space

# &#x20;           for vpn in area.vpn\_range {

# &#x20;               let src\_ppn = user\_space.translate(vpn).unwrap().ppn();

# &#x20;               let dst\_ppn = memory\_set.translate(vpn).unwrap().ppn();

# &#x20;               dst\_ppn.get\_bytes\_array().copy\_from\_slice(src\_ppn.get\_bytes\_array());

# &#x20;           }

# &#x20;       }

# &#x20;       memory\_set

# &#x20;   }

# }

# 

# 接着，实现 TaskControlBlock::fork 来从父进程的进程控制块创建一份子进程的控制块。实现如下：

# 

# //os/src/task/task.rs

# impl TaskControlBlock {

# &#x20;       pub fn fork(self: \&Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {

# &#x20;       // ---- access parent PCB exclusively

# &#x20;       let mut parent\_inner = self.inner\_exclusive\_access();

# &#x20;       // copy user space(include trap context)

# &#x20;       let memory\_set = MemorySet::from\_existed\_user(

# &#x20;           \&parent\_inner.memory\_set

# &#x20;       );

# &#x20;       let trap\_cx\_ppn = memory\_set

# &#x20;           .translate(VirtAddr::from(TRAP\_CONTEXT).into())

# &#x20;           .unwrap()

# &#x20;           .ppn();

# &#x20;       // alloc a pid and a kernel stack in kernel space

# &#x20;       let pid\_handle = pid\_alloc();

# &#x20;       let kernel\_stack = KernelStack::new(\&pid\_handle);

# &#x20;       let kernel\_stack\_top = kernel\_stack.get\_top();

# &#x20;       let task\_control\_block = Arc::new(TaskControlBlock {

# &#x20;           pid: pid\_handle,

# &#x20;           kernel\_stack,

# &#x20;           inner: unsafe { UPSafeCell::new(TaskControlBlockInner {

# &#x20;               trap\_cx\_ppn,

# &#x20;               base\_size: parent\_inner.base\_size,

# &#x20;               task\_cx: TaskContext::goto\_trap\_return(kernel\_stack\_top),

# &#x20;               task\_status: TaskStatus::Ready,

# &#x20;               memory\_set,

# &#x20;               parent: Some(Arc::downgrade(self)),

# &#x20;               children: Vec::new(),

# &#x20;               exit\_code: 0,

# &#x20;           })},

# &#x20;       });

# &#x20;       // add child

# &#x20;       parent\_inner.children.push(task\_control\_block.clone());

# &#x20;       // modify kernel\_sp in trap\_cx

# &#x20;       // \*\*\*\* access children PCB exclusively

# &#x20;       let trap\_cx = task\_control\_block.inner\_exclusive\_access().get\_trap\_cx();

# &#x20;       trap\_cx.kernel\_sp = kernel\_stack\_top;

# &#x20;       // return

# &#x20;       task\_control\_block

# &#x20;       // ---- release parent PCB automatically

# &#x20;       // \*\*\*\* release children PCB automatically

# &#x20;   }

# }

# 

# 

# 然后，实现exec系统调用。

# 

# //os/src/task/task.rs

# 

# impl TaskControlBlock {

# &#x20;   

# &#x20;   pub fn exec(\&self, elf\_data: \&\[u8]) {

# &#x20;       // memory\_set with elf program headers/trampoline/trap context/user stack

# &#x20;       let (memory\_set, user\_sp, entry\_point) = MemorySet::from\_elf(elf\_data);

# &#x20;       let trap\_cx\_ppn = memory\_set

# &#x20;           .translate(VirtAddr::from(TRAP\_CONTEXT).into())

# &#x20;           .unwrap()

# &#x20;           .ppn();

# 

# &#x20;       // \*\*\*\* access inner exclusively

# &#x20;       let mut inner = self.inner\_exclusive\_access();

# &#x20;       // substitute memory\_set

# &#x20;       inner.memory\_set = memory\_set;

# &#x20;       // update trap\_cx ppn

# &#x20;       inner.trap\_cx\_ppn = trap\_cx\_ppn;

# &#x20;       // initialize trap\_cx

# &#x20;       let trap\_cx = inner.get\_trap\_cx();

# &#x20;       \*trap\_cx = TrapContext::app\_init\_context(

# &#x20;           entry\_point,

# &#x20;           user\_sp,

# &#x20;           KERNEL\_SPACE.exclusive\_access().token(),

# &#x20;           self.kernel\_stack.get\_top(),

# &#x20;           trap\_handler as usize,

# &#x20;       );

# &#x20;       // \*\*\*\* release inner automatically

# &#x20;   }

# }

# 

# 有了exec系统调用后，sys\_exec的实现就很容易理解了。Sys\_exec的实现还依赖于对页表的修改。

# 

# //os/src/mm/page\_table.rs

# 

# use super::{

# &#x20;   PhysAddr

# };

# use alloc::string::String;

# 

# impl PageTable {

# &#x20;   pub fn translate\_va(\&self, va: VirtAddr) -> Option<PhysAddr> {

# &#x20;       self.find\_pte(va.clone().floor())

# &#x20;           .map(|pte| {

# &#x20;               //println!("translate\_va:va = {:?}", va);

# &#x20;               let aligned\_pa: PhysAddr = pte.ppn().into();

# &#x20;               //println!("translate\_va:pa\_align = {:?}", aligned\_pa);

# &#x20;               let offset = va.page\_offset();

# &#x20;               let aligned\_pa\_usize: usize = aligned\_pa.into();

# &#x20;               (aligned\_pa\_usize + offset).into()

# &#x20;           })

# &#x20;   }

# }

# 

# pub fn translated\_str(token: usize, ptr: \*const u8) -> String {

# &#x20;   let page\_table = PageTable::from\_token(token);

# &#x20;   let mut string = String::new();

# &#x20;   let mut va = ptr as usize;

# &#x20;   loop {

# &#x20;       let ch: u8 = \*(page\_table.translate\_va(VirtAddr::from(va)).unwrap().get\_mut());

# &#x20;       if ch == 0 {

# &#x20;           break;

# &#x20;       } else {

# &#x20;           string.push(ch as char);

# &#x20;           va += 1;

# &#x20;       }

# &#x20;   }

# &#x20;   string

# }

# 

# pub fn translated\_refmut<T>(token: usize, ptr: \*mut T) -> \&'static mut T {

# &#x20;   //println!("into translated\_refmut!");

# &#x20;   let page\_table = PageTable::from\_token(token);

# &#x20;   let va = ptr as usize;

# &#x20;   //println!("translated\_refmut: before translate\_va");

# &#x20;   page\_table.translate\_va(VirtAddr::from(va)).unwrap().get\_mut()

# }

# 

# 同时，需要修改os/src/mm/address.rs和os/src/mm/mod.rs的内容。

# 

# //os/src/mm/address.rs

# 

# impl PhysAddr {

# &#x20;   pub fn get\_mut<T>(\&self) -> \&'static mut T {

# &#x20;       unsafe {

# &#x20;           (self.0 as \*mut T).as\_mut().unwrap()

# &#x20;       }

# &#x20;   }

# }

# 

# //os/src/mm/mod.rs

# 

# pub use page\_table::{

# &#x20;   PageTableEntry,

# &#x20;   translated\_byte\_buffer,

# &#x20;   translated\_str,

# &#x20;   translated\_refmut,

# };

# 

# 在sys\_exec系统调用后，trap\_handler原来的上下文cx失效了。为此，在syscall分发之后，还需要重新获取trap上下文。具体修改代码如下：

# 

# //os/src/trap/mod.rs

# 

# 

# \#\[no\_mangle]

# pub fn trap\_handler() -> ! {

# &#x20;   set\_kernel\_trap\_entry();

# &#x20;   let cx = current\_trap\_cx();

# &#x20;   let scause = scause::read();

# &#x20;   let stval = stval::read();

# &#x20;   match scause.cause() {

# &#x20;       Trap::Exception(Exception::UserEnvCall) => {

# &#x20;           // jump to next instruction anyway

# &#x20;           let mut cx = current\_trap\_cx();

# &#x20;           cx.sepc += 4;

# &#x20;           // get system call return value

# &#x20;           let result = syscall(cx.x\[17], \[cx.x\[10], cx.x\[11], cx.x\[12]]);

# &#x20;           // cx is changed during sys\_exec, so we have to call it again

# &#x20;           cx = current\_trap\_cx();

# &#x20;           cx.x\[10] = result as usize;

# &#x20;       }

# &#x20;       Trap::Exception(Exception::StoreFault) |

# &#x20;       Trap::Exception(Exception::StorePageFault) |

# &#x20;       Trap::Exception(Exception::InstructionFault) |

# &#x20;       Trap::Exception(Exception::InstructionPageFault) |

# &#x20;       Trap::Exception(Exception::LoadFault) |

# &#x20;       Trap::Exception(Exception::LoadPageFault) => {

# &#x20;           println!(

# &#x20;               "\[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.",

# &#x20;               scause.cause(),

# &#x20;               stval,

# &#x20;               current\_trap\_cx().sepc,

# &#x20;           );

# &#x20;           // page fault exit code

# &#x20;           exit\_current\_and\_run\_next(-2);

# &#x20;       }

# &#x20;       Trap::Exception(Exception::IllegalInstruction) => {

# &#x20;           println!("\[kernel] IllegalInstruction in application, core dumped.");

# &#x20;           // illegal instruction exit code

# &#x20;           exit\_current\_and\_run\_next(-3);

# &#x20;       }

# &#x20;       Trap::Interrupt(Interrupt::SupervisorTimer) => {

# &#x20;           set\_next\_trigger();

# &#x20;           suspend\_current\_and\_run\_next();

# &#x20;       }

# &#x20;       \_ => {

# &#x20;           panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);

# &#x20;       }

# &#x20;   }

# &#x20;   trap\_return();

# }

# 

# 

# 11\. 进程资源回收机制

# 

# 当应用调用 sys\_exit 系统调用主动退出或者出错由内核终止之后，会在内核中调用 exit\_current\_and\_run\_next 函数退出当前进程并切换到下一个进程。

# 

# 相比之前的实现，exit\_current\_and\_run\_next增加了一个退出码作为参数。

# 注意修改/os/src/syscall/process.rs调用exit\_current\_and\_run\_next的部分。

# 

# // os/src/mm/memory\_set.rs

# 

# impl MemorySet {

# &#x20;   pub fn recycle\_data\_pages(\&mut self) {

# &#x20;       self.areas.clear();

# &#x20;   }

# }

# 

# //os/src/task/mod.rs

# 

# pub fn exit\_current\_and\_run\_next(exit\_code: i32) {

# &#x20;   // take from Processor

# &#x20;   let task = take\_current\_task().unwrap();

# &#x20;   // \*\*\*\* access current TCB exclusively

# &#x20;   let mut inner = task.inner\_exclusive\_access();

# &#x20;   // Change status to Zombie

# &#x20;   inner.task\_status = TaskStatus::Zombie;

# &#x20;   // Record exit code

# &#x20;   inner.exit\_code = exit\_code;

# &#x20;   // do not move to its parent but under initproc

# 

# &#x20;   // ++++++ access initproc TCB exclusively

# &#x20;   {

# &#x20;       let mut initproc\_inner = INITPROC.inner\_exclusive\_access();

# &#x20;       for child in inner.children.iter() {

# &#x20;           child.inner\_exclusive\_access().parent = Some(Arc::downgrade(\&INITPROC));

# &#x20;           initproc\_inner.children.push(child.clone());

# &#x20;       }

# &#x20;   }

# &#x20;   // ++++++ release parent PCB

# 

# &#x20;   inner.children.clear();

# &#x20;   // deallocate user space

# &#x20;   inner.memory\_set.recycle\_data\_pages();

# &#x20;   drop(inner);

# &#x20;   // \*\*\*\* release current PCB

# &#x20;   // drop task manually to maintain rc correctly

# &#x20;   drop(task);

# &#x20;   // we do not have to save task context

# &#x20;   let mut \_unused = TaskContext::zero\_init();

# &#x20;   schedule(\&mut \_unused as \*mut \_);

# }

# 

# 同时，父进程通过 sys\_waitpid 系统调用来回收子进程的资源并收集它的一些信息。

# 

# 最后，修改main.rs。

# 

# //os/src/main.rs

# 

# \#\[no\_mangle]

# pub fn rust\_main() -> ! {

# &#x20;   clear\_bss();

# &#x20;   println!("\[kernel] Hello, world!");

# &#x20;   mm::init();

# &#x20;   println!("\[kernel] back to world!");

# &#x20;   mm::remap\_test();

# &#x20;   task::add\_initproc();

# &#x20;   println!("after initproc!");

# &#x20;   trap::init();

# &#x20;   trap::enable\_timer\_interrupt();

# &#x20;   timer::set\_next\_trigger();

# &#x20;   loader::list\_apps();

# &#x20;   task::run\_tasks();

# &#x20;   panic!("Unreachable in rust\_main!");

# }

# 

# 

# 至此，具有进程管理功能的操作系统实现完成。

# 

# 12\. 思考并回答问题

# （1）分析应用的链接与加载是如何实现的；

# （2）分析进程标识符、进程控制块是如何设计和实现的；

# （3）分析任务管理是如何实现的；

# （4）分析进程的调度、生成、以及进程资源的回收是如何实现的。

# 

# 

