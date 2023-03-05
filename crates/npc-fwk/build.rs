fn main() {
    println!("cargo:rustc-link-lib=fwk");
    if let Ok(asan) = std::env::var("ASAN_LIBS") {
        println!("cargo:rustc-link-lib={asan}");
    }

    // This is expected to be set by the mesonb.build.
    if let Ok(libpath) = std::env::var("NIEPCE_LIB_PATH") {
        libpath.split(':').for_each(|s| {
            println!("cargo:rustc-link-search={s}");
        });
    }

    // Direct dependencies by the C++ code.
    pkg_config::Config::new()
        .atleast_version("4.4.0")
        .probe("gtkmm-4.0")
        .unwrap();
}
