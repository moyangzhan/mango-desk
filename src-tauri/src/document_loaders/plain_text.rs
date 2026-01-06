use crate::traits::document_loader::DocumentLoader;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

#[derive(Debug)]
pub struct PlainTextLoader {
    exts: Vec<String>,
}

impl Default for PlainTextLoader {
    fn default() -> Self {
        Self {
            exts: vec![
                "txt".to_string(),
                "md".to_string(),
                "log".to_string(),
                "mdx".to_string(),
            ],
        }
    }
}

impl DocumentLoader for PlainTextLoader {
    fn get_exts(&self) -> &[String] {
        &self.exts
    }

    fn add_ext(&mut self, ext: String) {
        self.exts.push(ext);
    }

    fn load(&self, path: &Path) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        return Ok(contents);
    }

    fn load_max(&self, path: &Path, max_load_chars: usize) -> io::Result<String> {
        let file = File::open(path)?;
        return self.load_file_max(&file, max_load_chars);
    }

    fn load_file_max(&self, file: &File, max_load_chars: usize) -> io::Result<String> {
        let mut contents = String::new();
        file.take((max_load_chars * 4) as u64)
            .read_to_string(&mut contents)?;
        let contents = contents.chars().take(max_load_chars).collect::<String>();
        return Ok(contents);
    }
}
