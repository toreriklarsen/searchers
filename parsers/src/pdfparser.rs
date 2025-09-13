use lopdf::Document;
use std::path::Path;
use log::debug;

pub fn read_pdf_text(path: &Path) -> std::result::Result<String, Box<dyn std::error::Error>> {
    debug!(" read_pdf_text");

    let doc = Document::load(path)?;
    let page_numbers = doc.get_pages().keys().cloned().collect::<Vec<_>>();

    let text = doc.extract_text(&page_numbers)?;

    Ok(text)
}