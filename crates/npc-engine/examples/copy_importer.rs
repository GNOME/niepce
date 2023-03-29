use std::path::PathBuf;

use clap::Parser;

use npc_engine::importer::{DatePathFormat, Importer};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    dest: String,

    #[arg(short, long)]
    source: String,

    #[arg(short, long)]
    recursive: bool,

    #[arg(short = 'n', long)]
    dry_run: bool,
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    println!("destination: {}", args.dest);

    let dest = PathBuf::from(args.dest);
    let source = PathBuf::from(args.source);
    let dry_run = args.dry_run;
    let verbose = args.verbose;
    let format = DatePathFormat::YearSlashYearMonthDay;

    let importer = Importer::from_dir(&source).set_recursive(args.recursive);
    if verbose {
        println!("Collecting files to import...");
    }
    let imports = importer.get_imports(&dest, format);
    for import in imports {
        if import.1.exists() {
            println!("{:?} already exists", import.1);
            continue;
        }
        if !dry_run {
            if verbose {
                println!("Copying {:?} to {:?}", import.0, import.1);
            }
            std::fs::create_dir_all(import.1.parent().expect("No parent, bailing out."))
                .expect("Couldn't create directories");
            npc_fwk::utils::copy(import.0, import.1).expect("Couldn't copy files.");
        } else {
            println!("Will copy {:?} to {:?}", import.0, import.1);
        }
    }
}
