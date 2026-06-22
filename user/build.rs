use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let linker_script = manifest_dir.join("src").join("linker.ld");
    println!("cargo:rustc-link-arg=-T{}", linker_script.display());
}
