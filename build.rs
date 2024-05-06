use std::path::PathBuf;

fn main() {
    let datasketches_src = PathBuf::from("datasketches-cpp");
    let rust_src = PathBuf::from("src");

    let mut bridge = cxx_build::bridge(rust_src.join("bridge.rs"));

    println!(
        "cargo:rerun-if-changed={}",
        rust_src.join("bridge.rs").to_str().unwrap()
    );
    println!(
        "cargo:rerun-if-changed={}",
        datasketches_src.join("cpc.cpp").to_str().unwrap()
    );
    println!(
        "cargo:rerun-if-changed={}",
        datasketches_src.join("theta.cp").to_str().unwrap()
    );
    println!(
        "cargo:rerun-if-changed={}",
        datasketches_src.join("hh.cpp").to_str().unwrap()
    );

    assert!(bridge.is_flag_supported("-std=c++11").expect("supported"));
    bridge
        .files(&[
            datasketches_src.join("cpc.cpp"),
            datasketches_src.join("theta.cpp"),
            datasketches_src.join("hh.cpp"),
        ])
        .include(datasketches_src.join("common").join("include"))
        .flag_if_supported("-std=c++11")
        .cpp_link_stdlib("stdc++")
        .static_flag(true)
        .compile("libdatasketches.a");
}
