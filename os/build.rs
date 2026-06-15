use std::fs::{read_dir, File};
use std::io::{Result, Write};
use std::path::PathBuf;

fn main() -> Result<()> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

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

    // 剥离 ELF 头，转为 raw binary（使用绝对路径）
    let user_dir = manifest_dir.join("../user/target/riscv64gc-unknown-none-elf/release");
    let user_dir_str = user_dir.to_str().unwrap();
    for app in &apps {
        let elf = format!("{}/{}", user_dir_str, app);
        let bin = format!("{}/{}.bin", user_dir_str, app);
        println!("cargo:warning=Stripping ELF: {} -> {}", elf, bin);
        elf_to_bin(&elf, &bin);
    }

    let mut f = File::create(manifest_dir.join("src/linkage.S")).unwrap();

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

    for (i, app) in apps.iter().enumerate() {
        println!("cargo:rerun-if-changed={}", manifest_dir.join(format!("../user/src/bin/{}.rs", app)).display());
        writeln!(f, r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
app_{0}_start:
    .incbin "{1}/{2}.bin"
app_{0}_end:"#, i, user_dir_str, app)?;
    }

    Ok(())
}

/// 解析 ELF64 文件，合并所有 PT_LOAD 段为 raw binary
fn elf_to_bin(elf_path: &str, bin_path: &str) {
    let data = std::fs::read(elf_path).expect("Failed to read ELF file");

    // 验证 ELF64 大端格式
    assert!(&data[0..4] == b"\x7fELF", "Not an ELF file");
    assert!(data[4] == 2, "Not 64-bit ELF");
    assert!(data[5] == 1, "Not little-endian");

    let phoff = read_u64(&data, 32) as usize;
    let phnum = read_u16(&data, 56) as usize;
    let phentsize = read_u16(&data, 54) as usize;

    // 收集所有 PT_LOAD 段: (vaddr, file_offset, filesz, memsz)
    let mut segments: Vec<(usize, usize, usize, usize)> = Vec::new();
    for i in 0..phnum {
        let off = phoff + i * phentsize;
        let p_type = read_u32(&data, off);
        if p_type == 1 {
            // PT_LOAD
            let p_offset = read_u64(&data, off + 8) as usize;
            let p_vaddr = read_u64(&data, off + 16) as usize;
            let p_filesz = read_u64(&data, off + 32) as usize;
            let p_memsz = read_u64(&data, off + 40) as usize;
            segments.push((p_vaddr, p_offset, p_filesz, p_memsz));
        }
    }

    assert!(!segments.is_empty(), "No PT_LOAD segments in ELF");

    // 计算输出 binary 的范围
    let min_vaddr = segments.iter().map(|s| s.0).min().unwrap();
    let max_end = segments.iter().map(|s| s.0 + s.3).max().unwrap();
    let total_size = max_end - min_vaddr;

    let mut bin = vec![0u8; total_size];
    for (vaddr, offset, filesz, _memsz) in &segments {
        let dst_start = vaddr - min_vaddr;
        bin[dst_start..dst_start + filesz]
            .copy_from_slice(&data[*offset..*offset + filesz]);
    }

    std::fs::write(bin_path, &bin).expect("Failed to write binary");
}

fn read_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

fn read_u64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ])
}
