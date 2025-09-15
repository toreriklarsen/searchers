//mod filetyper;

use walkdir::DirEntry;
use log::debug;
use std::env;
use std::time::SystemTime;
use url::Url;
use serde::Serialize;
use parsers::{pdfparser::read_pdf_text, docxparser::read_docx_text};
use crate::filetyper::{get_file_type, SupportedFileType};

#[derive(Serialize, Debug)]
struct SearchDocument {
    id: String,
    filename: String,
    url: Url,
    content: Option<String>,
    created: Option<SystemTime>,
    modified: Option<SystemTime>,
    size: Option<u64>,
    filetype: Option<String>,
}

fn process_file(entry: DirEntry) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let attr = entry.metadata()?;
    let path = entry.path().to_owned();
    let filename = path.to_string_lossy().to_string();
    let doctype: SupportedFileType = get_file_type(&path);

    if doctype == SupportedFileType::Shouldneverhappen {
        debug!("Skipping unsupported file: {}", filename);
        return Ok(());
    }
    debug!(
        "Spawning task for file: {}, doctype {:?}",
        filename, doctype
    );

    let text = match &doctype {
        SupportedFileType::Pdf => read_pdf_text(&path),
        SupportedFileType::Docx => read_docx_text(&path),
        SupportedFileType::Shouldneverhappen => Err("This should not happen".into()),
    };

    let full_path = env::current_dir().unwrap().join(&path);
    //debug!("Full file path {}", &full_path.to_string_lossy());
    
    match text {
        Ok(content) => Some(SearchDocument {
            id: format!("{:x}", md5::compute(&filename)),
            filename,
            url: Url::from_file_path(full_path)
                .unwrap_or_else(|_| Url::parse("file:///unknown").unwrap()),
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
    };
    
    Ok(())
}
