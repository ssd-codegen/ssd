use std::env;
use std::path::PathBuf;

#[cfg(feature = "c_parser")]
fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let libdir_path = manifest_dir
        .join("..")
        .join("..")
        .join("extern")
        .join("minissd");

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

    let mut build = cc::Build::new();

    #[cfg(not(target_os = "windows"))]
    build.static_flag(true);

    #[cfg(target_os = "windows")]
    build.static_crt(true);

    build
        .include(libdir_path.join("include"))
        .file(libdir_path.join("src").join("minissd.c"))
    .compile("minissd");

    println!("cargo:rustc-link-lib=static=minissd");
}

#[cfg(not(feature = "c_parser"))]
fn main() {}
