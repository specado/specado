use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_dir = PathBuf::from(&crate_dir).join("include");
    
    // Create include directory if it doesn't exist
    std::fs::create_dir_all(&output_dir).unwrap();
    
    // Generate the header file using cbindgen.toml config
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(output_dir.join("specado.h"));
    
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/api.rs");
    println!("cargo:rerun-if-changed=src/types.rs");
    println!("cargo:rerun-if-changed=src/memory.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");
}