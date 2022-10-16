use std::env;
use std::path::PathBuf;

fn main() {
    if env::var("SKIP_CBINDINGS").is_err() {
        // Use cbindgen to generate C bindings.
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| String::from("./target"));
        let mut target_file = PathBuf::from(target_dir);
        target_file.push("eng_bindings.h");
        let mut config = cbindgen::Config::default();
        config.constant.allow_static_const = false;
        cbindgen::Builder::new()
            .with_config(config)
            .with_include_guard("niepce_rust_eng_bindings_h")
            .with_namespace("ffi")
            .with_language(cbindgen::Language::Cxx)
            .with_parse_deps(true)
            .with_parse_exclude(&[
                "exempi", "chrono", "multimap", "clap", "nom", "winapi", "glib", "rusqlite",
                "strum",
            ])
            .include_item("Managed")
            .include_item("NiepcePropertyIdx")
            .exclude_item("CUSTOM_START")
            .exclude_item("INTERNAL_START")
            .exclude_item("GdkPixbuf")
            .exclude_item("GtkWindow")
            .exclude_item("GtkToolbar")
            .exclude_item("GFileInfo")
            .exclude_item("RgbColour")
            .exclude_item("PropertyValue")
            .exclude_item("Keyword")
            .exclude_item("Label")
            .exclude_item("LibFile")
            // Ensure these are opaque as generics are still a problem.
            .exclude_item("NiepcePropertySet")
            .exclude_item("NiepcePropertyBag")
            .exclude_item("PropertySet")
            .exclude_item("PropertyBag")
            .exclude_item("Option")
            .with_crate(&crate_dir)
            .generate()
            .expect("Couldn't generate bindings")
            .write_to_file(&*target_file.to_string_lossy());
    }
}
