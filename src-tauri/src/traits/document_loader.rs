use std::io;
use std::path::Path;

pub trait DocumentLoader {
    fn get_exts(&self) -> &[String];
    fn add_ext(&mut self, ext: String);
    fn load(&self, path: &Path) -> io::Result<String>;

    /// Loads content from a file up to a maximum number of chars(not characters).
    ///
    /// # Arguments
    /// * `path` - A reference to the Path that points to the file to be loaded
    /// * `max_load_chars` - The maximum number of chars to read from the file.
    ///   This value serves as an approximate limit - if the loaded content exceeds this
    ///   size, loading stops and the currently loaded content is returned.
    ///
    /// # Returns
    /// * `io::Result<String>` - Returns the loaded content as a String, or an io::Error if the operation fails
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// let content = my_doc_loader.load_max(Path::new("example.txt"), 1000)?;
    /// ```
    fn load_max(&self, path: &Path, max_load_chars: usize) -> io::Result<String>;
    fn load_file_max(&self, file: &std::fs::File, max_load_chars: usize) -> io::Result<String>;
}

pub trait OpenOfficeDoc {
    fn load<P: AsRef<Path>>(path: P, max_load_chars: usize) -> io::Result<String>;
    fn load_max<P: AsRef<Path>>(path: P, max_load_chars: usize) -> io::Result<String>;
}
