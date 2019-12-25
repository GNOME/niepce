extern crate cbindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    if env::var("SKIP_CBINDINGS").is_err() {
        // Use cbindgen to generate C bindings.
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or(String::from("./target"));
        let mut target_file = PathBuf::from(target_dir);
        target_file.push("fwk_bindings.h");
        let cbuilder = cbindgen::Builder::new()
            .with_include_guard("niepce_fwk_rust_bindings_h")
            .with_namespace("ffi")
            .with_language(cbindgen::Language::Cxx)
            .with_parse_deps(true)
            .with_parse_exclude(&["exempi", "chrono", "multimap"])
            .exclude_item("GtkWindow")
            .exclude_item("GtkToolbar")
            .exclude_item("GFileInfo")
            .with_crate(&crate_dir);

        if let Ok(bindings) = cbuilder.generate() {
            bindings.write_to_file(&*target_file.to_string_lossy());
        } else {
            println!("Couldn't generate bindings");
        }
    }
}