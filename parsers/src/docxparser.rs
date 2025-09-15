use log::debug;
use std::fs::File;
use std::io::Read;
use std::path::Path;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    #[test]
    fn test_read_docx_text_simple() {
        //let path = create_docx_with_text("Hello, world!");

        let options = SimpleFileOptions::default().unix_permissions(0o755);
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("multi.docx");
        let file = File::create(&file_path).unwrap();
        let mut zip = ZipWriter::new(file);

        let document_xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
            r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                <w:body>
                    <w:p>
                        <w:r>
                            <w:t>Hello, world!</w:t>
                        </w:r>
                    </w:p>
                </w:body>
            </w:document>"#;

        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();
        zip.finish().unwrap();



        let result = read_docx_text(&file_path);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Hello, world!"));
    }

    #[test]
    fn test_read_docx_text_multiple_text_nodes() {
        let options = SimpleFileOptions::default().unix_permissions(0o755);
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("multi.docx");
        let file = File::create(&file_path).unwrap();
        let mut zip = ZipWriter::new(file);

        let document_xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
            <w:body>
                <w:p>
                    <w:r><w:t>First</w:t></w:r>
                    <w:r><w:t>Second</w:t></w:r>
                    <w:r><w:t>Third</w:t></w:r>
                </w:p>
            </w:body>
        </w:document>"#;

        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();
        zip.finish().unwrap();

        let result = read_docx_text(&file_path).unwrap();
        assert!(result.contains("First"));
        assert!(result.contains("Second"));
        assert!(result.contains("Third"));
    }

    #[test]
    fn test_read_docx_text_missing_document_xml() {
        let options = SimpleFileOptions::default().unix_permissions(0o755);
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("missing.docx");
        let file = File::create(&file_path).unwrap();
        let mut zip = ZipWriter::new(file);

        zip.start_file("word/other.xml", options).unwrap();
        zip.write_all(b"<xml></xml>").unwrap();
        zip.finish().unwrap();

        let result = read_docx_text(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_docx_text_empty_document_xml() {
        let options = SimpleFileOptions::default().unix_permissions(0o755);
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("empty.docx");
        let file = File::create(&file_path).unwrap();
        let mut zip = ZipWriter::new(file);

        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(b"").unwrap();
        zip.finish().unwrap();

        let result = read_docx_text(&file_path).unwrap();
        assert_eq!(result.trim(), "");
    }
}
