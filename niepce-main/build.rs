fn main() {
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
