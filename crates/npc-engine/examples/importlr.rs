/*
 * niepce - bin/importlr.rs
 *
 * Copyright (C) 2021-2025 Hubert Figuière
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

use clap::Parser;
use serde_derive::Deserialize;

use std::io::Read;
use std::path::{Path, PathBuf};

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

#[derive(Parser, Debug)]
/// Import a Lr catalog
struct Args {
    #[arg(short, action=clap::ArgAction::Count)]
    /// Sets the level of verbosity
    verbosity: u8,

    #[arg(short = 'n', long)]
    /// Dry run
    dry_run: bool,

    #[arg(short, long)]
    /// File containing roots remap
    roots: Option<String>,

    #[arg(short = 'L')]
    /// Which library to import into
    library: String,

    /// The catalog to import
    catalog: String,
}

fn main() {
    npc_fwk::init();

    let args = Args::parse();

    let library = &args.library;
    let catalog = &args.catalog;
    let verbosity = args.verbosity;

    let (sender, _recv) = async_channel::unbounded();

    let library = LibraryClient::new(PathBuf::from(library), sender);
    // library.init();
    let mut importer = LrImporter::new();
    if !LrImporter::can_import_library(Path::new(catalog)) {
        println!("Can't import catalog {}", catalog);
        return;
    }

    importer
        .init_importer(Path::new(catalog))
        .expect("Init importer");

    if let Some(roots) = &args.roots {
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
