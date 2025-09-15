//#![allow(unused_imports)]
mod cli;
mod filetyper;

use chrono::Local;
use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;
use log::{debug};
use parsers::{docxparser::read_docx_text, pdfparser::read_pdf_text};
use serde::Serialize;
use url::Url;
use std::env;
use std::io::Write;
use meilisearch_sdk::{client::Client, indexes::Index};
use tokio::task;
use walkdir::{DirEntry, WalkDir};
use filetyper::*;

#[derive(Serialize, Debug)]
struct SearchDocument {
    id: String,
    filename: String,
    url: Url,
    content: Option<String>,
    created: Option<std::time::SystemTime>,
    modified: Option<std::time::SystemTime>,
    size: Option<u64>,
    filetype: Option<String>,
}

fn is_not_skipped(entry: &DirEntry) -> bool {
    let file_name = entry.file_name().to_string_lossy();
    // Skip directories named "target" or ".git"
    !(file_name == "target" || file_name == ".git")
}



#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
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

    debug!("indexer options {:?}", clia);

    // 2. Walk a folder for documents
    let folder = &clia.inputdir;
    let mut tasks = Vec::new();

    for entry in WalkDir::new(folder).into_iter().filter_entry(is_not_skipped).filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let attr = entry.metadata()?;
            let path = entry.path().to_owned();
            let filename = path.to_string_lossy().to_string();
            let doctype: SupportedFileType = get_file_type(&path);
            
            if doctype == SupportedFileType::Shouldneverhappen {
                debug!("Skipping unsupported file: {}", filename);
                continue;
            }
            debug!("Spawning task for file: {}, doctype {:?}", filename, doctype);
            
            // Start extraction in a new threads
            let handle = task::spawn_blocking(move || {
                let text = match &doctype {
                    SupportedFileType::Pdf => read_pdf_text(&path),
                    SupportedFileType::Docx => read_docx_text(&path),
                    SupportedFileType::Shouldneverhappen => Err("This should not happen".into())
                };

                let full_path = env::current_dir().unwrap().join(&path);
                debug!("Full file path {}", &full_path.to_string_lossy());
                match text {
                    Ok(content) => Some(SearchDocument {
                        id: format!("{:x}", md5::compute(&filename)),
                        filename,
                        url: Url::from_file_path(full_path).unwrap_or_else(|_| Url::parse("file:///unknown").unwrap()),
                        content: content.into(),
                        created: attr.created().ok(),
                        modified: attr.modified().ok(),
                        size: attr.len().into(),
                        filetype: if let Some(ext) = path.extension() {
                            ext.to_string_lossy().to_string().into()
                        } else {
                            "unknown".to_string().into()
                        },
                    }),
                    Err(_) => None,
                }
            });

            tasks.push(handle);
        }
    }

    // 3. Collect results from worker threads
    let mut docs = Vec::new();
    for task in tasks {
        if let Ok(Some(doc)) = task.await {
            debug!("Prepared document: {:?}", &doc);
            docs.push(doc);
        }
    }

    // 4. Push to Meilisearch
    if !docs.is_empty() {
        debug!("Indexing {} documents", docs.len());
        index.add_documents(&docs, Some("id")).await?;
        println!("Indexed {} documents!", docs.len());
    } else {
        println!("No documents indexed.");
    }

    Ok(())
}
