extern crate cbindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    if env::var("SKIP_CBINDINGS").is_err() {
        // Use cbindgen to generate C bindings.
        let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| String::from("./target"));
        let mut target_file = PathBuf::from(target_dir);
        target_file.push("bindings.h");
        let mut config = cbindgen::Config::default();
        config.constant.allow_static_const = false;
        cbindgen::Builder::new()
            .with_config(config)
            .with_include_guard("niepce_rust_bindings_h")
            .with_namespace("ffi")
            .with_language(cbindgen::Language::Cxx)
            .with_parse_deps(true)
            .with_parse_exclude(&[
                "exempi", "chrono", "multimap", "glib", "clap", "winapi", "rusqlite", "nom",
                "strum",
            ])
            .include_item("ColIndex")
            .include_item("ItemTypes")
            .exclude_item("Managed")
            .exclude_item("NiepcePropertyIdx")
            .exclude_item("FileStatus")
            .exclude_item("RgbColour")
            .exclude_item("CUSTOM_START")
            .exclude_item("INTERNAL_START")
            .exclude_item("GdkPixbuf")
            .exclude_item("GtkWindow")
            .exclude_item("GtkToolbar")
            .exclude_item("GtkTreeIter")
            .exclude_item("GtkTreeModel")
            .exclude_item("GtkTreeView")
            .exclude_item("GtkIconView")
            .exclude_item("GtkListStore")
            .exclude_item("GtkTreePath")
            .exclude_item("GtkPopoverMenu")
            .exclude_item("GtkMessageDialog")
            .exclude_item("GtkCellRenderer")
            .exclude_item("GtkWidget")
            .exclude_item("GFileInfo")
            .exclude_item("PortableChannel")
            .exclude_item("Option")
            .with_crate(&crate_dir)
            .generate()
            .expect("Couldn't generate bindings")
            .write_to_file(&target_file);
    }

    #[cfg(examples)]
    {
        use std::fs::remove_file;
        use std::path::Path;
        use std::process::Command;
        use std::str::from_utf8;

        // Remove old versions of the gresource to make sure we're using the latest version
        if Path::new("examples/gresource.gresource").exists() {
            remove_file("examples/gresource.gresource").unwrap();
        }

        // Compile Gresource
        let mut source_dir = String::from("--sourcedir=");
        source_dir.push_str(&crate_dir);
        let mut target_dir = String::from("--target=");
        let mut target_path = PathBuf::from(crate_dir);
        target_path.push("examples");
        target_path.push("gresource.gresource");
        target_dir.push_str(target_path.to_str().unwrap());
        let output =
            Command::new(option_env!("GRESOURCE_BINARY_PATH").unwrap_or("glib-compile-resources"))
                .args(&[
                    "--generate",
                    "gresource.xml",
                    source_dir.as_str(),
                    target_dir.as_str(),
                ])
                .current_dir("src/niepce")
                .output()
                .unwrap();

        if !output.status.success() {
            println!("Failed to generate GResources!");
            println!(
                "glib-compile-resources stdout: {}",
                from_utf8(&output.stdout).unwrap()
            );
            println!(
                "glib-compile-resources stderr: {}",
                from_utf8(&output.stderr).unwrap()
            );
            panic!("Can't continue build without GResources!");
        }
    }
}
