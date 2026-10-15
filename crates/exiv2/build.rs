/*
 * exiv2-rs - build.rs
 *
 * Copyright (C) 2026 Hubert Figuière
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

fn main() {
    let deps = system_deps::Config::new().probe().unwrap();
    let exiv2 = deps.get_by_name("exiv2").unwrap();

    let includes = exiv2
        .include_paths
        .iter()
        .map(|p| format!("-I{}", p.to_string_lossy()))
        .collect::<Vec<_>>();
    let bindings = bindgen::Builder::default()
        .clang_args(["-x", "c++", "-std=gnu++17"])
        .clang_args(&includes)
        // We want the enums to be safe (new type) and enum-ish
        .default_enum_style(bindgen::EnumVariation::NewType {
            is_bitfield: false,
            is_global: false,
        })
        .allowlist_type("Exiv2::ImageType")
        .allowlist_type("Exiv2::TypeId")
        .allowlist_type("Exiv2::ByteOrder")
        .header("./src/npc_bindings.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    cxx_build::bridge("src/lib.rs") // returns a cc::Build
        .include("./src")
        .includes(&exiv2.include_paths)
        .flag("-std=c++17")
        .compile("exiv2-rs");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/npc_exiv2.h");
    println!("cargo:rerun-if-changed=src/npc_bindings.h");
}
