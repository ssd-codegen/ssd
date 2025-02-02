use std::env;
use std::path::PathBuf;

#[cfg(feature = "c_parser")]
fn main() {
    let libdir_path = PathBuf::from("../../extern/minissd")
        .canonicalize()
        .expect("cannot canonicalize path");

    println!(
        "cargo:rerun-if-changed={}",
        libdir_path
            .join("include")
            .join("minissd.h")
            .to_str()
            .unwrap()
    );
    println!(
        "cargo:rerun-if-changed={}",
        libdir_path.join("src").join("minissd.c").to_str().unwrap()
    );

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    cc::Build::new()
        .static_flag(true)
        // .define("DEBUG_HAPPY_PATH", None)
        .include(libdir_path.join("include"))
        .file(libdir_path.join("src").join("minissd.c"))
        .out_dir(&out_dir)
        .compile("minissd");

    println!("cargo:rustc-link-search={}", out_dir.to_str().unwrap());

    println!("cargo:rustc-link-lib=minissd");
}

#[cfg(not(feature = "c_parser"))]
fn main() {}
