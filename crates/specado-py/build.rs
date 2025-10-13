use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn copy_providers(from: &Path, to: &Path) -> io::Result<()> {
    if to.exists() {
        fs::remove_dir_all(to)?;
    }
    fs::create_dir_all(to)?;

    fn copy_dir(src: &Path, dst: &Path) -> io::Result<()> {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if file_type.is_dir() {
                fs::create_dir_all(&dst_path)?;
                copy_dir(&src_path, &dst_path)?;
            } else if file_type.is_file() {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    copy_dir(from, to)
}

fn main() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cargo manifest dir"));
    let source = manifest_dir.join("../specado-providers/providers");
    let destination = manifest_dir.join("../../python/specado/providers");

    println!("cargo:rerun-if-changed={}", source.display());

    copy_providers(&source, &destination)
}
