extern crate cbindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    if env::var("SKIP_CBINDINGS").is_err() {
        // Use cbindgen to generate C bindings.
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| String::from("./target"));
        let mut target_file = PathBuf::from(target_dir);
        target_file.push("fwk_bindings.h");
        let mut config = cbindgen::Config::default();
        config.constant.allow_static_const = false;
        cbindgen::Builder::new()
            .with_config(config)
            .with_include_guard("niepce_fwk_rust_bindings_h")
            .with_namespace("ffi")
            .with_language(cbindgen::Language::Cxx)
            .with_parse_deps(true)
            .with_parse_exclude(&[
                "exempi", "chrono", "multimap", "glib", "clap", "winapi", "strum",
            ])
            .exclude_item("Date")
            .exclude_item("CUSTOM_START")
            .exclude_item("INTERNAL_START")
            .exclude_item("GdkPixbuf")
            .exclude_item("GtkDrawingArea")
            .exclude_item("GtkWidget")
            .exclude_item("GtkWindow")
            .exclude_item("GtkToolbar")
            .exclude_item("GFileInfo")
            .with_crate(&crate_dir)
            .generate()
            .expect("Unable to generate bindings")
            .write_to_file(&target_file);
    }
}
