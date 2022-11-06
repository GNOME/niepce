/*
 * niepce - bin/importlr.rs
 *
 * Copyright (C) 2021-2022 Hubert Figuière
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

//!
//! Sample command line tool to import a Lr Catalog
//!

use clap::{App, Arg};
use serde_derive::Deserialize;

use std::io::Read;
use std::path::PathBuf;

use npc_engine::importer::LrImporter;
use npc_engine::importer::{LibraryImporter, LibraryImporterProbe};
use npc_engine::libraryclient::LibraryClient;

///
/// The remaps as loaded from toml passed with `-r`
///
/// ```toml
/// roots = [
///     [ "origin1", "dest1" ],
///     [ "origin2", "dest2" ]
/// ]
/// ```
///
#[derive(Debug, Deserialize)]
struct Remaps {
    roots: Vec<Vec<String>>,
}

fn main() {
    npc_fwk::init();

    let matches = App::new("LrImporter")
        .version("0.1.0")
        .about("Import a Lr catalog")
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            Arg::with_name("CATALOG")
                .help("The catalog to import")
                .required(true),
        )
        .arg(
            Arg::with_name("roots")
                .short("r")
                .value_name("ROOTS")
                .help("File containing roots remap")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("library")
                .short("L")
                .value_name("LIBRARY")
                .help("Which library to import into")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let library = matches.value_of("library").unwrap();
    let catalog = matches.value_of("CATALOG").unwrap();
    let verbosity = matches.occurrences_of("v");

    let (sender, _recv) = async_channel::unbounded();

    let library = LibraryClient::new(PathBuf::from(library), sender);
    // library.init();
    let mut importer = LrImporter::new();
    if !LrImporter::can_import_library(&PathBuf::from(catalog)) {
        println!("Can't import catalog {}", catalog);
        return;
    }

    importer
        .init_importer(&PathBuf::from(catalog))
        .expect("Init importer");

    if let Some(roots) = matches.value_of("roots") {
        let mut file = std::fs::File::open(roots).expect("Can't open roots file");
        let mut content = String::new();
        file.read_to_string(&mut content).expect("Can't read roots");

        if let Ok(remaps) = toml::from_str::<Remaps>(&content) {
            remaps.roots.iter().for_each(|v| {
                if v.len() >= 2 {
                    importer.map_root_folder(&v[0], &v[1]);
                }
            });
        } else {
            println!("Invalid roots file");
        }
    }

    let before = std::time::Instant::now();
    importer.import_library(&library).expect("Import Library");
    println!("Elapsed time: {:.2?}", before.elapsed());
}
