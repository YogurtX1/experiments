use std::fs::{read_dir, File};
use std::io::{Result, Write};
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=user/src/");
    println!("cargo:rerun-if-changed=user/target/");
    insert_app_data().unwrap();
}

fn insert_app_data() -> Result<()> {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("linkage.S");
    let mut f = File::create(&dest_path)?;

    // 🎯 修复点：去掉 read_dir(...)? 后面的 .unwrap()
    let mut apps: Vec<_> = read_dir("user/target/riscv64gc-unknown-none-elf/release/")?
        .filter_map(|dir_entry| {
            let entry = dir_entry.ok()?;
            let path = entry.path();
            let file_name = path.file_name()?.to_str()?;
            if path.is_file() && file_name.ends_with(".bin") {
                Some(file_name.to_string())
            } else {
                None
            }
        })
        .collect();

    apps.sort();

    writeln!(
        f,
        r#"
    .section .data
    .global _num_app
_num_app:
    .quad {}"#,
        apps.len()
    )?;

    for i in 0..apps.len() {
        writeln!(f, "    .quad app_{}_start", i)?;
    }
    writeln!(f, "    .quad app_{}_end", apps.len() - 1)?;

    for (idx, app) in apps.iter().enumerate() {
        println!("cargo:rustc-env=APP_{}={}", idx, app);
        writeln!(
            f,
            r#"
    .section .data
    .global app_{idx}_start
    .global app_{idx}_end
app_{idx}_start:
    .incbin "user/target/riscv64gc-unknown-none-elf/release/{app}"
app_{idx}_end:"#,
            idx = idx,
            app = app
        )?;
    }
    Ok(())
}
