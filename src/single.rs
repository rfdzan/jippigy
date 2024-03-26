use crate::compress::Compress;
use colored::Colorize;
use std::{
    io,
    path::{Path, PathBuf},
};
/// Single image compressions.
pub struct Single<T: AsRef<Path>> {
    path: T,
    quality: u8,
    dir: PathBuf,
    default_prefix: Option<String>,
}
impl<T: AsRef<Path>> Single<T> {
    /// Creates new single image task.
    pub fn new(path: T, quality: u8, dir: PathBuf, prefix: Option<String>) -> Self
    where
        T: AsRef<Path>,
    {
        Self {
            path,
            quality,
            dir,
            default_prefix: prefix,
        }
    }
    /// Compress a single image.
    pub fn do_single(self) -> io::Result<()> {
        self.exists().compress();
        Ok(())
    }
    /// Check whether or not image exists.
    fn exists(self) -> Self {
        if !self.path.as_ref().exists() {
            eprintln!(
                "File does not exist: {}. Maybe there's a {}?\nInclude the extension {} as well.",
                self.path
                    .as_ref()
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
                    .red(),
                "typo".to_string().yellow(),
                ".jpg/.JPG/.jpeg/.JPEG".to_string().green()
            );
            std::process::exit(1);
        }
        self
    }
    /// Compress a single image.
    fn compress(self) {
        let prefix = match self.default_prefix {
            None => "".to_string(),
            Some(p) => p,
        };
        match Compress::compress(self.path, self.dir, self.quality, Some(prefix)) {
            Err(e) => eprintln!("{e}"),
            Ok(msg) => println!("{msg}"),
        }
    }
}
