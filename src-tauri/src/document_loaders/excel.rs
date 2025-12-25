use crate::traits::document_loader::DocumentLoader;
use calamine::Reader;
use calamine::open_workbook_auto;
use std::io;
use std::path::Path;

#[derive(Debug)]

pub struct ExcelLoader {
    exts: Vec<String>,
}

impl Default for ExcelLoader {
    fn default() -> Self {
        Self {
            exts: vec![
                "xlsx".to_string(),
                "xls".to_string(),
                "xlsm".to_string(),
                "xlsb".to_string(),
                "xla".to_string(),
                "xlam".to_string(),
                "ods".to_string(),
            ],
        }
    }
}

impl DocumentLoader for ExcelLoader {
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
        let mut workbook = open_workbook_auto(path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Failed to open Excel file: {}", e),
            )
        })?;
        let sheet_names = workbook.sheet_names();
        let mut txt = String::new();
        'outer: for sheet_name in sheet_names {
            let range = workbook.worksheet_range(&sheet_name).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Failed to open Excel file: {}", e),
                )
            })?;
            if range.rows().len() == 0 {
                continue;
            } else {
                for row in range.rows() {
                    for cell in row {
                        txt.push_str(&cell.to_string());
                        txt.push_str("\t");
                        if max_load_chars > 0 && txt.chars().count() > max_load_chars {
                            break 'outer;
                        }
                    }
                }
            }
        }
        Ok(txt)
    }
    fn load_file_max(&self, file: &std::fs::File, max_load_chars: usize) -> io::Result<String> {
        unimplemented!(
            "load_file_max with File is not supported for Excel files, use load_max() with Path instead"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instantiate() {
        let _ = ExcelLoader::default().load_max(Path::new("samples/sample.xlsx"), 1000);
    }

    #[test]
    fn read() {
        let data = ExcelLoader::default()
            .load_max(Path::new("samples/sample.xlsx"), 1000)
            .unwrap();

        println!("len: {},data: {}", data.len(), data);
    }
}
