use std::fs::{read_dir, File};
use std::io::{Result, Write};
use std::path::PathBuf;

fn main() -> Result<()> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    // 传递链接脚本
    let linker_script = manifest_dir.join("src").join("linker.ld");
    println!("cargo:rustc-link-arg=-T{}", linker_script.display());

    // 从 ../user/src/bin 读取应用
    let mut apps: Vec<_> = read_dir(manifest_dir.join("../user/src/bin"))
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    apps.sort();

    let user_dir = manifest_dir.join("../user/target/riscv64gc-unknown-none-elf/release");
    let user_dir_str = user_dir.to_str().unwrap();

    let mut f = File::create(manifest_dir.join("src/linkage.S")).unwrap();

    // _num_app and app start/end pointers
    writeln!(
        f,
        r#"
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}"#,
        apps.len()
    )?;

    for (i, _app) in apps.iter().enumerate() {
        writeln!(f, r#"
    .quad app_{}_start"#, i)?;
    }
    // 终止符：最后一个应用的结束地址
    writeln!(f, "\n    .quad app_{}_end", apps.len() - 1)?;

    // _app_names: 所有应用名字以 null 结尾顺序排列
    writeln!(f, r#"
    .global _app_names
_app_names:"#)?;
    for app in apps.iter() {
        writeln!(f, r#"    .string "{}""#, app)?;
    }

    // 嵌入每个应用的完整 ELF 文件（保留 ELF 头供内核 from_elf 解析）
    for (i, app) in apps.iter().enumerate() {
        println!("cargo:rerun-if-changed={}", manifest_dir.join(format!("../user/src/bin/{}.rs", app)).display());
        writeln!(f, r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
app_{0}_start:
    .incbin "{1}/{2}"
app_{0}_end:"#, i, user_dir_str, app)?;
    }

    Ok(())
}
