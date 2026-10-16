fn main() {
    println!("cargo:rustc-link-lib=rtengine");
    // This is for gcc, there is a different one for clang.
    println!("cargo:rustc-link-lib=gomp");
    if let Ok(asan) = std::env::var("ASAN_LIBS") {
        println!("cargo:rustc-link-lib={asan}");
    }

    let deps = system_deps::Config::new().probe().unwrap();
    let glibmm = deps.get_by_name("glibmm-2.68").unwrap();
    let giomm = deps.get_by_name("giomm-2.68").unwrap();
    let exiv2 = deps.get_by_name("exiv2").unwrap();

    let build_root = std::path::PathBuf::from(
        std::env::var("CARGO_TARGET_DIR").expect("CARGO_TARGET_DIR not found"),
    )
    .join("..");
    let rtengine_path = build_root.join("third_party/rtengine");

    cxx_build::bridge("src/bridge.rs") // returns a cc::Build
        .file("src/npc_rtengine.cpp")
        .include("../../third_party/rtengine/RawTherapee")
        .include("./src")
        .include(&rtengine_path)
        .includes(&glibmm.include_paths)
        .includes(&giomm.include_paths)
        .includes(&exiv2.include_paths)
        // rtengine header is full of this.
        .flag("-DUSE_STD_MUTEX=1")
        .flag("-DNPC_NOGUI=1")
        .flag("-Wno-unused-parameter")
        .flag("-std=c++17")
        .compile("librtbridge");

    let mut includes = glibmm
        .include_paths
        .iter()
        .map(|p| format!("-I{}", p.to_string_lossy()))
        .collect::<Vec<_>>();
    includes.push("-I../../third_party/rtengine/RawTherapee".to_string());
    let bindings = bindgen::Builder::default()
        .clang_args(["-x", "c++", "-std=gnu++17"])
        .clang_args(["-DNPC_NOGUI=1", "-DNPC_BINDGEN=1"])
        .clang_args(&includes)
        // We want the enums to be safe (new type) and enum-ish
        .default_enum_style(bindgen::EnumVariation::NewType {
            is_bitfield: false,
            is_global: false,
        })
        .allowlist_type("rtengine::procparams::LcMode")
        .header("./src/npc_rtengine.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=src/bridge.rs");
    println!("cargo:rerun-if-changed=src/npc_rtengine.h");
    println!("cargo:rerun-if-changed=src/npc_rtengine.cpp");
    let rtconfig = rtengine_path.join("npc_rtconfig.h");
    println!("cargo:rerun-if-changed={}", rtconfig.to_string_lossy());
    println!(
        "cargo:rustc-link-search={}",
        rtengine_path.to_string_lossy()
    );
}
