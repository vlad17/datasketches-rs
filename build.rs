use std::path::PathBuf;

fn main() {
    let datasketches = PathBuf::from("datasketches-cpp");
    let src = PathBuf::from("src");
    let mut bridge = cxx_build::bridge(src.join("bridge.rs"));

    assert!(bridge.is_flag_supported("-std=c++14").expect("supported"));
    bridge
        .files(&[
            datasketches.join("cpc.cpp"),
            datasketches.join("theta.cpp"),
            datasketches.join("hh.cpp"),
            datasketches.join("kll.cpp"),
        ])
        .include(datasketches.join("common").join("include"))
        .flag_if_supported("-std=c++14")
        .cpp_link_stdlib("stdc++")
        .static_flag(true)
        .compile("libdatasketches.a");
}
