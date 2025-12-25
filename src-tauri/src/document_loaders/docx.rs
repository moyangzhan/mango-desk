use crate::traits::document_loader::DocumentLoader;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug)]
pub struct DocxLoader {
    exts: Vec<String>,
}

impl Default for DocxLoader {
    fn default() -> Self {
        Self {
            exts: vec!["docx".to_string()],
        }
    }
}

impl DocumentLoader for DocxLoader {
    fn get_exts(&self) -> &[String] {
        &self.exts
    }

    fn add_ext(&mut self, ext: String) {
        self.exts.push(ext);
    }

    fn load(&self, path: &Path) -> io::Result<String> {
        let file = File::open(path)?;
        self.load_file_max(&file, 0)
    }

    fn load_max(&self, path: &Path, max_load_chars: usize) -> io::Result<String> {
        let file = File::open(path)?;
        self.load_file_max(&file, max_load_chars)
    }

    fn load_file_max(&self, file: &File, max_load_chars: usize) -> io::Result<String> {
        let mut archive = ZipArchive::new(file)?;

        let mut xml_data = String::new();

        for i in 0..archive.len() {
            let mut c_file = archive.by_index(i).unwrap();
            if c_file.name() == "word/document.xml" {
                let read_result = c_file.read_to_string(&mut xml_data)?;
                if read_result == 0 {
                    println!("Error reading file")
                }
                break;
            }
        }

        let mut xml_reader = Reader::from_str(xml_data.as_ref());
        xml_reader.config_mut().trim_text(true);

        let mut txt = Vec::new();
        if xml_data.len() > 0 {
            loop {
                match xml_reader.read_event() {
                    Ok(Event::Start(ref e)) => match e.name().as_ref() {
                        b"w:p" => {
                            txt.push("\n\n".to_string());
                        }
                        _ => (),
                    },
                    Ok(Event::Text(e)) => {
                        let decode = e.decode().map_err(|e| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("XML decode error: {}", e),
                            )
                        })?;
                        txt.push(decode.into_owned());

                        let total_chars: usize = txt.iter().map(|s| s.chars().count()).sum();
                        if total_chars > max_load_chars {
                            break;
                        }
                    }
                    Ok(Event::Eof) => break, // exits the loop when reaching end of file
                    Err(e) => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!(
                                "Error at position {}: {:?}",
                                xml_reader.buffer_position(),
                                e
                            ),
                        ));
                    }
                    _ => (),
                }
            }
        }
        Ok(txt.join(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instantiate() {
        let _ = DocxLoader::default().load_max(Path::new("assets/test_file/example.docx"), 1000);
    }

    #[test]
    fn read() {
        let data = DocxLoader::default()
            .load_max(Path::new("assets/test_file/example.docx"), 1000)
            .unwrap();
        println!("len: {}, data: {}", data.len(), data);
    }
}
