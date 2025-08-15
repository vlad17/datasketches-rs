use std::path::PathBuf;

fn main() {
    let datasketches = PathBuf::from("datasketches-cpp");
    let src = PathBuf::from("src");
    let mut bridge = cxx_build::bridge(src.join("bridge.rs"));

    assert!(bridge.is_flag_supported("-std=c++11").expect("supported"));

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    let stdlib = if target_os == "macos" {
        "c++"
    } else {
        "stdc++"
    };

    bridge
        .files(&[
            datasketches.join("cpc.cpp"),
            datasketches.join("theta.cpp"),
            datasketches.join("hh.cpp"),
        ])
        .include(datasketches.join("common").join("include"))
        .flag_if_supported("-std=c++11")
        .cpp_link_stdlib(stdlib)
        .static_flag(true)
        .compile("libdatasketches.a");
}
