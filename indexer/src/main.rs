//#![allow(unused_imports)]
use clap::Parser;
use env_logger::Builder;
use log::debug;
use chrono::Local;
use parsers::{docxparser::read_docx_text, pdfparser::read_pdf_text};
use serde::Serialize;
use std::io::Write;
mod cli;
use meilisearch_sdk::{client::Client, indexes::Index};
use tokio::task;
use walkdir::WalkDir;
use log::LevelFilter;
#[derive(Serialize, Debug)]
struct SearchDocument {
    id: String,
    filename: String,
    content: String,
}


#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // 1. Connect to Meilisearch
    let client = Client::new("http://127.0.0.1:7700", Some("masterKey"))?;
    let index: Index = client.index("documents");

    Builder::new()
        .filter(None, LevelFilter::Debug) // Default log level set to Debug
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"), // Timestamp
                record.level(),                           // Log level
                record.args()                             // Log message
            )
        })
        .init();

    let clia = cli::Cli::parse();

    println!("indexer! {:?}", clia);
    debug!("This is a debug message");

    // 2. Walk a folder for documents
    let folder = &clia.inputdir;
    let mut tasks = Vec::new();

    for entry in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path().to_owned();
            let filename = path.to_string_lossy().to_string();

            let handle = task::spawn_blocking(move || {
                let text = if filename.ends_with(".docx") {
                    read_docx_text(&path)
                } else if filename.ends_with(".pdf") {
                    read_pdf_text(&path)
                } else {
                    return None;
                };

                match text {
                    Ok(content) => Some(SearchDocument {
                        id: format!("{:x}", md5::compute(&filename)),
                        filename,
                        content,
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
            docs.push(doc);
        }
    }

    // 4. Push to Meilisearch
    if !docs.is_empty() {
        index.add_documents(&docs, Some("id")).await?;
        println!("Indexed {} documents!", docs.len());
    } else {
        println!("No documents indexed.");
    }

    Ok(())
}
