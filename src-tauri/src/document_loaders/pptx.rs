use crate::global::PPTX_EXTS;
use crate::traits::document_loader::DocumentLoader;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug)]
pub struct PptxLoader {
    exts: Vec<String>,
}

impl Default for PptxLoader {
    fn default() -> Self {
        Self {
            exts: PPTX_EXTS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl DocumentLoader for PptxLoader {
    fn get_exts(&self) -> &[String] {
        &self.exts
    }
    fn add_ext(&mut self, ext: String) {
        self.exts.push(ext);
    }
    fn load(&self, path: &Path) -> io::Result<String> {
        self.load_max(path, 0)
    }
    fn load_max(&self, path: &Path, max_load_chars: usize) -> io::Result<String> {
        let file = File::open(path)?;
        self.load_file_max(&file, max_load_chars)
    }
    fn load_file_max(&self, file: &std::fs::File, max_load_chars: usize) -> io::Result<String> {
        let mut archive = ZipArchive::new(file)?;

        let mut xml_data = String::new();

        for i in 0..archive.len() {
            let mut c_file = archive.by_index(i).unwrap();
            if c_file.name().starts_with("ppt/slides") {
                let mut _buff = String::new();
                let read_result = c_file.read_to_string(&mut _buff)?;
                if read_result == 0 {
                    println!("Error reading file")
                }
                xml_data += _buff.as_str();
            }
        }

        let mut txt = Vec::new();

        if xml_data.len() > 0 {
            let mut to_read = false;
            let mut xml_reader = Reader::from_str(xml_data.as_ref());
            loop {
                match xml_reader.read_event() {
                    Ok(Event::Start(ref e)) => match e.name().as_ref() {
                        b"a:p" => {
                            to_read = true;
                            txt.push("\n".to_string());
                        }
                        b"a:t" => {
                            to_read = true;
                        }
                        _ => (),
                    },
                    Ok(Event::Text(e)) => {
                        if to_read {
                            let decode = e.decode().map_err(|e| {
                                io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!("XML decode error: {}", e),
                                )
                            })?;
                            txt.push(decode.into_owned());
                            to_read = false;

                            if max_load_chars < 1 {
                                continue;
                            }
                            let total_chars: usize = txt.iter().map(|s| s.chars().count()).sum();
                            if total_chars > max_load_chars {
                                break;
                            }
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

mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn instantiate() {
        let _ = PptxLoader::default().load_max(Path::new("test/sample.pptx"), 1000);
    }

    #[test]
    fn read() {
        let data = PptxLoader::default()
            .load_max(Path::new("test/sample.pptx"), 1000)
            .unwrap();
        println!("len: {}, data: {}", data.len(), data);
    }
}
