use crate::traits::document_loader::DocumentLoader;
use lopdf::{Document, Object};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::io;
use std::io::{Error, ErrorKind};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct PdfText {
    text: BTreeMap<u32, Vec<String>>, // Key is page number
    errors: Vec<String>,
}

#[derive(Debug)]
pub struct PdfLoader {
    exts: Vec<String>,
}

static IGNORE: &[&[u8]] = &[
    b"Length",
    b"BBox",
    b"FormType",
    b"Matrix",
    b"Type",
    b"XObject",
    b"Subtype",
    b"Filter",
    b"ColorSpace",
    b"Width",
    b"Height",
    b"BitsPerComponent",
    b"Length1",
    b"Length2",
    b"Length3",
    b"PTEX.FileName",
    b"PTEX.PageNumber",
    b"PTEX.InfoDict",
    b"FontDescriptor",
    b"ExtGState",
    b"MediaBox",
    b"Annot",
];

fn filter_func(object_id: (u32, u16), object: &mut Object) -> Option<((u32, u16), Object)> {
    if IGNORE.contains(&object.type_name().unwrap_or_default()) {
        return None;
    }
    if let Ok(d) = object.as_dict_mut() {
        d.remove(b"Producer");
        d.remove(b"ModDate");
        d.remove(b"Creator");
        d.remove(b"ProcSet");
        d.remove(b"Procset");
        d.remove(b"XObject");
        d.remove(b"MediaBox");
        d.remove(b"Annots");
        if d.is_empty() {
            return None;
        }
    }
    Some((object_id, object.to_owned()))
}

fn load_pdf<P: AsRef<Path>>(path: P) -> Result<Document, Error> {
    Document::load_filtered(path, filter_func)
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
}

fn load_form(file: &std::fs::File) -> Result<Document, Error> {
    Document::load_from(file).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
}

fn get_pdf_text(doc: &Document, max_load_chars: usize) -> Result<PdfText, Error> {
    let mut pdf_text: PdfText = PdfText {
        text: BTreeMap::new(),
        errors: Vec::new(),
    };
    let mut loaded_chars = 0;
    let pages: Vec<Result<(u32, Vec<String>), Error>> = doc
        .get_pages()
        .into_iter()
        .scan(max_load_chars, |max_load, (page_num, page_id)| {
            if loaded_chars >= *max_load {
                return None;
            }
            let text = match doc.extract_text(&[page_num]) {
                Ok(t) => t,
                Err(e) => {
                    return Some(Err(Error::new(
                        ErrorKind::Other,
                        format!("Failed to extract text from page {page_num} id={page_id:?}: {e:}"),
                    )));
                }
            };
            loaded_chars += text.chars().count();
            Some(Ok((
                page_num,
                text.split('\n')
                    .map(|s| s.trim_end().to_string())
                    .collect::<Vec<String>>(),
            )))
        })
        .collect();
    for page in pages {
        match page {
            Ok((page_num, lines)) => {
                pdf_text.text.insert(page_num, lines);
            }
            Err(e) => {
                pdf_text.errors.push(e.to_string());
            }
        }
    }
    Ok(pdf_text)
}

impl Default for PdfLoader {
    fn default() -> Self {
        Self {
            exts: vec!["pdf".to_string()],
        }
    }
}

impl DocumentLoader for PdfLoader {
    fn get_exts(&self) -> &[String] {
        &self.exts
    }

    fn add_ext(&mut self, ext: String) {
        self.exts.push(ext);
    }

    fn load(&self, path: &Path) -> io::Result<String> {
        println!("Load:{} ", path.display());
        let mut doc = load_pdf(&path)?;
        if doc.is_encrypted() {
            doc.decrypt(&"")
                .map_err(|_err| Error::new(ErrorKind::InvalidInput, "Failed to decrypt"))?;
        }
        let text = get_pdf_text(&doc, usize::MAX)?;
        if !text.errors.is_empty() {
            eprintln!("{} has {} errors:", path.display(), text.errors.len());
            for error in &text.errors[..10] {
                eprintln!("{error:?}");
            }
        }
        let contents = text
            .text
            .values()
            .flat_map(|lines| lines.iter())
            .cloned()
            .collect::<String>();
        Ok(contents)
    }

    fn load_max(&self, path: &Path, max_load_chars: usize) -> io::Result<String> {
        println!("Load:{} ", path.display());
        let mut doc = load_pdf(&path)?;
        if doc.is_encrypted() {
            doc.decrypt(&"")
                .map_err(|_err| Error::new(ErrorKind::InvalidInput, "Failed to decrypt"))?;
        }
        let pdf_text = get_pdf_text(&doc, max_load_chars)?;
        if !pdf_text.errors.is_empty() {
            eprintln!("{} has {} errors:", path.display(), pdf_text.errors.len());
            for error in &pdf_text.errors[..10] {
                eprintln!("{error:?}");
            }
        }
        let contents = pdf_text
            .text
            .values()
            .flat_map(|lines| lines.iter())
            .cloned()
            .collect::<String>();
        Ok(contents)
    }

    fn load_file_max(&self, _file: &std::fs::File, _max_load_chars: usize) -> io::Result<String> {
        Err(Error::new(
            ErrorKind::Unsupported,
            "Loading from file is not supported for PDF",
        ))
    }
}
