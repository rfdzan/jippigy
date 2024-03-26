use crate::{compress::Compress, TaskArgs};
use colored::Colorize;
use std::{env, io, path::PathBuf};
/// Single image compressions.
pub struct Single {
    args: TaskArgs,
    cur_dir: PathBuf,
    full_path: PathBuf,
    default_prefix: String,
}
impl Single {
    /// Creates new single image task.
    pub fn new(args: TaskArgs) -> Self {
        Self {
            args,
            cur_dir: PathBuf::new(),
            full_path: PathBuf::new(),
            default_prefix: "smoljpg_".to_string(),
        }
    }
    /// Compress a single image.
    pub fn do_single(self) -> io::Result<()> {
        self.prep()?.exists().compress();
        Ok(())
    }
    /// Check whether or not image exists.
    fn exists(self) -> Self {
        if !self.full_path.exists() {
            eprintln!(
                "File does not exist: {}. Maybe there's a {}?\nInclude the extension {} as well.",
                self.full_path
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
        match Compress::compress(
            self.full_path,
            self.cur_dir,
            self.args.get_quality(),
            Some(self.default_prefix),
        ) {
            Err(e) => eprintln!("{e}"),
            Ok(msg) => println!("{msg}"),
        }
    }
    /// Get image information.
    fn prep(mut self) -> io::Result<Self> {
        let cur_dir = env::current_dir()?;
        let filename = self.args.get_single();
        let full_path = cur_dir.join(filename);
        self.full_path = full_path;
        self.cur_dir = cur_dir;
        Ok(self)
    }
}
