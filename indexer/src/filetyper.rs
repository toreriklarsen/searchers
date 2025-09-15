use infer::{self};
use log::{debug};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum SupportedFileType {
    Pdf,
    Docx,
    Shouldneverhappen,
}

pub fn get_file_type(path: &Path) -> SupportedFileType {
    let filename = path.to_string_lossy().to_string();

    let fileextension = match path.extension() {
        Some(ext) => ext.to_string_lossy().to_string().to_lowercase(),
        None => "none".to_string(),
    };

    let mut doctype = SupportedFileType::Shouldneverhappen;

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return SupportedFileType::Shouldneverhappen,
    };

    let mut buffer = [0u8; 512]; // Read only the first 512 bytes
    if file.read(&mut buffer).is_err() {
        debug!("Could not read file: {}", filename);
        return SupportedFileType::Shouldneverhappen;
    }

    match infer::get(&buffer) {
        Some(info) => {
            debug!(
                "Assume file: {} is of type: {} with extension: {:?}",
                filename,
                info.mime_type(),
                path.extension()
            );
            doctype = match info.mime_type() {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                    debug!("File: {} identified as DOCX", filename);
                    SupportedFileType::Docx
                }
                "application/pdf" => {
                    debug!("File: {} identified as PDF", filename);
                    SupportedFileType::Pdf
                }
                "application/zip" => {
                    if fileextension == "docx" {
                        debug!("File: {} identified as DOCX", filename);
                        SupportedFileType::Docx
                    } else {
                        debug!("File: {} is a ZIP but not a DOCX", filename);
                        SupportedFileType::Shouldneverhappen
                    }
                }
                _ => {
                    debug!(
                        "File: {} is of unknown mimetype {:?}",
                        filename,
                        info.mime_type()
                    );
                    SupportedFileType::Shouldneverhappen
                }
            }
        }
        None => {
            debug!("File: {} is of unknown type", filename);
        } //Err(e) => {
          //  warn!("Error determining file type for {}: {}", filename, e);
          //}
    }

    doctype
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    use std::fs::File;

    fn write_bytes_to_file(path: &std::path::Path, bytes: &[u8]) {
        let mut file = File::create(path).unwrap();
        file.write_all(bytes).unwrap();
    }

    #[test]
    fn test_pdf_file_type() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.pdf");
        // PDF files start with "%PDF"
        write_bytes_to_file(&file_path, b"%PDF-1.4\n");
        assert_eq!(get_file_type(&file_path), SupportedFileType::Pdf);
    }

    #[test]
    fn test_docx_file_type_by_mime() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.docx");
        // DOCX files are ZIP files with specific structure, but infer detects by magic number
        // ZIP magic number: PK\x03\x04
        write_bytes_to_file(&file_path, b"PK\x03\x04");
        assert_eq!(get_file_type(&file_path), SupportedFileType::Docx);
    }

    #[test]
    fn test_zip_file_not_docx() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.zip");
        write_bytes_to_file(&file_path, b"PK\x03\x04");
        assert_eq!(
            get_file_type(&file_path),
            SupportedFileType::Shouldneverhappen
        );
    }

    #[test]
    fn test_unknown_file_type() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.unknown");
        write_bytes_to_file(&file_path, b"randomdata");
        assert_eq!(
            get_file_type(&file_path),
            SupportedFileType::Shouldneverhappen
        );
    }

    #[test]
    fn test_file_without_extension() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("no_extension");
        write_bytes_to_file(&file_path, b"PK\x03\x04");
        assert_eq!(
            get_file_type(&file_path),
            SupportedFileType::Shouldneverhappen
        );
    }

    #[test]
    fn test_nonexistent_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("does_not_exist.pdf");
        assert_eq!(
            get_file_type(&file_path),
            SupportedFileType::Shouldneverhappen
        );
    }
}
