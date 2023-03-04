fn main() {
    println!("cargo:rustc-link-lib=ncr");

    // This is expected to be set by the mesonb.build.
    if let Ok(libpath) = std::env::var("NIEPCE_LIB_PATH") {
        libpath.split(':').for_each(|s| {
            println!("cargo:rustc-link-arg=-L{s}");
        });
    }
}
