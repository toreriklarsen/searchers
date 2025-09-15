use log::debug;
use lopdf::Document;
use std::path::PathBuf;

pub fn read_pdf_text(path: &PathBuf) -> std::result::Result<String, Box<dyn std::error::Error>> {
    debug!(" read_pdf_text");

    let doc = Document::load(path)?;
    let page_numbers = doc.get_pages().keys().cloned().collect::<Vec<_>>();
    let text = doc.extract_text(&page_numbers)?;

    Ok(text)
}

#[cfg(test)]
#[macro_use]
mod tests {
    use super::*;
    use lopdf::content::{Content, Operation};
    use lopdf::dictionary;
    use lopdf::{Document, Object, Stream};
    use std::fs::File;
    use std::io::Write;

    fn create_test_pdf(path: &PathBuf) {
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Courier",
        });
        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! {
                "F1" => font_id,
            },
        });
        let content = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 48.into()]),
                Operation::new("Td", vec![100.into(), 600.into()]),
                Operation::new("Tj", vec![Object::string_literal("Test PDF")]),
                Operation::new("ET", vec![]),
            ],
        };

        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        doc.compress();
        //doc.save("test.pdf").unwrap();
        doc.save(&path).unwrap();
    }

    #[test]
    fn test_read_pdf_text_success() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let pdf_path = tmp_dir.path().join("test.pdf");

        create_test_pdf(&pdf_path);

        let result = read_pdf_text(&pdf_path);
        eprintln!("After2 = {:?}", pdf_path.exists());
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("Test PDF"));
    }

    #[test]
    fn test_read_pdf_text_file_not_found() {
        let path = PathBuf::from("non_existent_file.pdf");
        let result = read_pdf_text(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_pdf_text_invalid_pdf() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let pdf_path = tmp_dir.path().join("invalid.pdf");
        let mut file = File::create(&pdf_path).unwrap();
        file.write_all(b"not a pdf").unwrap();

        let result = read_pdf_text(&pdf_path);
        assert!(result.is_err());
    }
}
