//#![allow(unused_imports)]
mod cli;
mod filetyper;
mod processfile;

use chrono::Local;
use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;
use log::debug;
use meilisearch_sdk::{client::Client, indexes::Index};
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use std::io::Write;
use walkdir::{DirEntry, WalkDir};

fn is_not_skipped(entry: &DirEntry) -> bool {
    let file_name = entry.file_name().to_string_lossy();
    !(file_name == "target" || file_name == ".git")
}

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // 1. Connect to Meilisearch
    let client = Client::new("http://127.0.0.1:7700", Some("masterKey"))?;
    let index: Index = client.index("documents");

    Builder::new()
        .filter(None, LevelFilter::Debug) // Default log level set to Debug
        .filter_module("lopdf", LevelFilter::Error)
        .format(|buf, record| {
            let tid = std::thread::current().id();
            writeln!(
                buf,
                "{} [{}][{:?}] - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                tid,
                record.args()
            )
        })
        .init();

    let clia = cli::Cli::parse();

    // 2. Walk a folder for documents
    let folder = &clia.inputdir;

    rayon::ThreadPoolBuilder::new()
        .num_threads(8)
        .build_global()
        .unwrap();

    let docs = WalkDir::new(folder)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(is_not_skipped)
        .map(|e| e.into_path())
        .par_bridge()
        .filter_map(processfile::process_file)
        .collect::<Vec<_>>();

    // 4. Push to Meilisearch
    if !clia.noindex && !docs.is_empty() {
        debug!("Indexing {} documents", docs.len());
        index.add_documents(&docs, Some("id")).await?;
        println!("Indexed {} documents!", docs.len());
    } else  {
        println!("No documents indexed.");
    }

    Ok(())
}
