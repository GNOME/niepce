use clap::{App, Arg};
use std::path::PathBuf;

use npc_engine::importer::LibraryImporter;
use npc_engine::importer::LrImporter;
use npc_engine::libraryclient::LibraryClient;

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

    let mut library = LibraryClient::new(PathBuf::from(library), sender);
    // library.init();
    let mut importer = LrImporter::new();
    if !LrImporter::can_import_library(&PathBuf::from(catalog)) {
        println!("Can't import catalog {}", catalog);
        return;
    }
    if importer.import_library(&PathBuf::from(catalog), &mut library) {}
}
