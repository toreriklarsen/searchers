use std::io::Read;
use std::path::Path;
use std::fs::File;
use log::debug;
use zip::read::ZipArchive;

pub fn read_docx_text(path: &Path) -> std::result::Result<String, Box<dyn std::error::Error>> {
    debug!(" read_docx_text");
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut doc_xml = archive.by_name("word/document.xml")?;
    let mut xml_content = String::new();
    doc_xml.read_to_string(&mut xml_content)?;

    // Extract <w:t> text nodes
    let text: String = xml_content
        .split("<w:t")
        .skip(1)
        .map(|chunk| match chunk.split_once('>') {
            Some((_, rest)) => rest.split_once('<').map(|(txt, _)| txt).unwrap_or(""),
            None => "",
        })
        .collect::<Vec<_>>()
        .join(" ");

    Ok(text)
}