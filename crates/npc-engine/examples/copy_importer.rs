use std::path::PathBuf;

use clap::Parser;

use npc_engine::importer::{DatePathFormat, Importer};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    dest: String,

    #[arg(short, long)]
    source: String,
}

fn main() {
    let args = Args::parse();

    println!("destination: {}", args.dest);

    let dest = PathBuf::from(args.dest);
    let source = PathBuf::from(args.source);
    let format = DatePathFormat::YearSlashYearMonthDay;

    let importer = Importer {};
    let imports = importer.get_imports(&source, &dest, format);
    for import in imports {
        println!("{:?} => {:?}", import.0, import.1);
    }
    //let dir = Importer::dest_dir_for_date(&dest, Date::now(), format);
}
