fn main() {
    println!("cargo:rustc-link-lib=ncr");
    println!("cargo:rustc-link-lib=fwk");

    // This is expected to be set by the mesonb.build.
    if let Ok(libpath) = std::env::var("NIEPCE_LIB_PATH") {
        libpath.split(':').for_each(|s| {
            println!("cargo:rustc-link-search={s}");
        });
    }

    pkg_config::Config::new()
        .atleast_version("0.4")
        .probe("gegl-0.4")
        .unwrap();
}
