use crate::SupportedFileType;
use infer::{self, doc};
use log::{debug, info, warn};
use std::path::Path;
use std::fs::File;
use std::io::Read;

pub fn get_file_type(path: &Path) -> SupportedFileType {
    let filename = path.to_string_lossy().to_string();

    let fileextension = match path.extension() {
        Some(ext) => ext.to_string_lossy().to_string().to_lowercase(),
        None => "none".to_string(),
    };

    let mut doctype = SupportedFileType::SHOULDNEVERHAPPEN;

    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(_) => return SupportedFileType::SHOULDNEVERHAPPEN,
    };

    let mut buffer = [0u8; 512]; // Read only the first 512 bytes
    if !file.read(&mut buffer).is_ok() {
        debug!("Could not read file: {}", filename);
        return SupportedFileType::SHOULDNEVERHAPPEN;
    }

    match infer::get(&buffer) {
        Some(info) => {
            debug!(
                "Assume file: {} is of type: {} with extension: {:?}",
                filename,
                info.mime_type().to_string(),
                path.extension()
            );
            doctype = match info.mime_type() {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                    debug!("File: {} identified as DOCX", filename);
                    SupportedFileType::DOCX
                }
                "application/pdf" => {
                    debug!("File: {} identified as PDF", filename);
                    SupportedFileType::PDF
                }
                "application/zip" => {
                    if fileextension == "docx" {
                        debug!("File: {} identified as DOCX", filename);
                        SupportedFileType::DOCX
                    } else {
                        debug!("File: {} is a ZIP but not a DOCX", filename);
                        SupportedFileType::SHOULDNEVERHAPPEN
                    }
                }
                _ => {
                    debug!(
                        "File: {} is of unknown mimetype {:?}",
                        filename,
                        info.mime_type()
                    );
                    SupportedFileType::SHOULDNEVERHAPPEN
                }
            }
        }
        None => {
            debug!("File: {} is of unknown type", filename);
        }
        //Err(e) => {
          //  warn!("Error determining file type for {}: {}", filename, e);
        //}
    }


    doctype
}

