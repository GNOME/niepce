/*
 * niepce - npc_engine/examples/copy_importer.rs
 *
 * Copyright (C) 2023-2025 Hubert Figuière
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

use std::path::PathBuf;

use clap::Parser;

use npc_engine::catalog::CatalogDb;
use npc_engine::importer::{DatePathFormat, Importer};
use npc_engine::library::commands::cmd_import_files;
use npc_engine::library::notification::LibNotification;
use npc_fwk::ExempiManager;

#[derive(Parser, Debug)]
struct Args {
    /// Destination base directory.
    #[arg(short, long)]
    dest: String,

    /// Source directory.
    #[arg(short, long)]
    source: String,

    /// (Optional) Catalog to import into.
    #[arg(short, long)]
    catalog: Option<String>,

    /// (Optional) Date directories to restrict. Pass mulitple times
    /// for multiple directories.
    #[arg(long)]
    date: Option<Vec<String>>,

    /// Import recursively
    #[arg(short, long)]
    recursive: bool,

    /// Dry run.
    #[arg(short = 'n', long)]
    dry_run: bool,
    /// Verbose output.
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    let _ = ExempiManager::new(None);

    println!("destination: {}", args.dest);

    let dest = PathBuf::from(&args.dest);
    let source = PathBuf::from(args.source);
    let catalog = args.catalog.map(PathBuf::from);
    let dry_run = args.dry_run;
    let verbose = args.verbose;
    let dates = args.date;
    let format = DatePathFormat::YearSlashYearMonthDay;

    if verbose {
        println!("Collecting files to import...");
    }
    let catalog = catalog.map(|file| {
        let (sender, receiver) = async_channel::unbounded();
        let catalog = CatalogDb::new(&file, sender);

        // Note that this could cause an infinite loop.
        while let Ok(msg) = receiver.try_recv() {
            match msg {
                LibNotification::LibCreated => println!("Database was created"),
                LibNotification::DatabaseReady => break,
                _ => {}
            }
        }

        catalog
    });
    let imports = Importer::get_imports(&source, &dest, format, args.recursive);
    let only_dest_dirs = dates.map(|dates| dates.iter().map(|d| dest.join(d)).collect::<Vec<_>>());
    let mut size_to_copy = 0;
    for import in &imports {
        if let Some(only_dest_dirs) = only_dest_dirs.as_ref() {
            if let Some(parent) = import.1.parent().map(|p| p.to_path_buf()) {
                if !only_dest_dirs.contains(&parent) {
                    println!("{:?} excluded", import.1);
                    continue;
                }
            }
        }
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
            npc_fwk::utils::copy(&import.0, &import.1).expect("Couldn't copy files.");
        } else {
            if let Ok(metadata) = std::fs::metadata(&import.0) {
                size_to_copy += metadata.len();
            }
            println!("Will copy {:?} to {:?}", import.0, import.1);
        }
    }
    if dry_run {
        println!(
            "Would have copied {}",
            humanize_bytes::humanize_bytes_decimal!(size_to_copy)
        );
    }
    if !dry_run {
        let imports: Vec<PathBuf> = imports.into_iter().map(|elem| elem.1).collect();
        if let Some(catalog) = &catalog {
            cmd_import_files(catalog, &dest, &imports);
        }
    }
}
