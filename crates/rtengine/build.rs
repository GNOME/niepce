fn main() {
    println!("cargo:rustc-link-lib=rtengine");
    // This is for gcc, there is a different one for clang.
    println!("cargo:rustc-link-lib=gomp");
    if let Ok(asan) = std::env::var("ASAN_LIBS") {
        println!("cargo:rustc-link-lib={asan}");
    }
    // This is expected to be set by the meson.build.
    if let Ok(libpath) = std::env::var("NIEPCE_LIB_PATH") {
        libpath.split(':').for_each(|s| {
            println!("cargo:rustc-link-search={s}");
        });
    }
    pkg_config::Config::new().probe("expat").unwrap();
    pkg_config::Config::new().probe("fftw3f").unwrap();
    let glibmm = pkg_config::Config::new().probe("glibmm-2.68").unwrap();
    pkg_config::Config::new().probe("giomm-2.68").unwrap();
    pkg_config::Config::new().probe("lcms2").unwrap();
    pkg_config::Config::new().probe("lensfun").unwrap();
    pkg_config::Config::new().probe("libiptcdata").unwrap();
    pkg_config::Config::new().probe("libjpeg").unwrap();
    pkg_config::Config::new().probe("libpng16").unwrap();
    pkg_config::Config::new().probe("libtiff-4").unwrap();
    pkg_config::Config::new().probe("zlib").unwrap();

    let build_root = std::path::PathBuf::from(
        std::env::var("CARGO_TARGET_DIR").expect("CARGO_TARGET_DIR not found"),
    )
    .join("..");

    cxx_build::bridge("src/bridge.rs") // returns a cc::Build
        .file("src/npc_rtengine.cpp")
        .include("../../third_party/rtengine/RawTherapee")
        .include("./src")
        .include(build_root.join("third_party/rtengine"))
        .includes(glibmm.include_paths)
        // rtengine header is full of this.
        .flag("-DUSE_STD_MUTEX=1")
        .flag("-DNPC_NOGUI=1")
        .flag("-Wno-unused-parameter")
        .flag("-std=c++17")
        .compile("librtbridge");

    println!("cargo:rerun-if-changed=src/bridge.rs");
    println!("cargo:rerun-if-changed=src/npc_rtengine.h");
    println!("cargo:rerun-if-changed=src/npc_rtengine.cpp");
    println!("cargo:rerun-if-changed={build_root:?}/third_party/rtengine/npc_rtconfig.h");
}
