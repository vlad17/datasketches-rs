use std::path::PathBuf;

fn main() {
    let datasketches = PathBuf::from("datasketches-cpp");
    let src = PathBuf::from("src");
    let mut bridge = cxx_build::bridge(src.join("bridge.rs"));

    assert!(bridge.is_flag_supported("-std=c++11").expect("supported"));
    bridge
        .file(datasketches.join("bridge.cpp"))
        .include(datasketches.join("common").join("include"))
        .flag_if_supported("-std=c++11")
        .cpp_link_stdlib("stdc++")
        .static_flag(true)
        // .warnings(false)
        .compile("libdatasketches.a");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/blobstore.cc");
    println!("cargo:rerun-if-changed=include/blobstore.h");
}
