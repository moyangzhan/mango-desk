use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use zip::ZipArchive;

pub(crate) fn read_data<P: AsRef<Path>>(
    path: P,
    content_name: &str,
    tags: &[&str],
    max_load_chars: usize,
) -> io::Result<String> {
    let file = File::open(path.as_ref())?;
    let mut archive = ZipArchive::new(file)?;

    let mut xml_data = String::new();

    for i in 0..archive.len() {
        let mut c_file = archive.by_index(i)?;
        if c_file.name() == content_name {
            let read_result = c_file.read_to_string(&mut xml_data)?;
            if read_result == 0 {
                println!("Error reading file")
            }
            break;
        }
    }

    let mut xml_reader = Reader::from_str(xml_data.as_ref());

    let mut txt = Vec::new();

    if xml_data.len() > 0 {
        let mut to_read = false;
        loop {
            match xml_reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    for tag in tags {
                        let name = e.name();
                        let name_ref = name.as_ref();
                        if name_ref == tag.as_bytes() {
                            to_read = true;
                            if name_ref == b"text:p" {
                                txt.push("\n\n".to_string());
                            }
                            break;
                        }
                    }
                }
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

                        if max_load_chars > 0 {
                            let total_chars: usize = txt.iter().map(|s| s.chars().count()).sum();
                            if total_chars > max_load_chars {
                                break;
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
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
