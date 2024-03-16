use crate::{compress::Compress, TaskArgs};
use colored::Colorize;
use std::{env, io};
/// Single image compressions.
pub struct Single {
    args: TaskArgs,
}
impl Single {
    /// Creates new single image task.
    pub fn new(args: TaskArgs) -> Self {
        Self { args }
    }
    /// Compress a single image.
    pub fn do_single(&self) -> io::Result<()> {
        let cur_dir = env::current_dir()?;
        let filename = self.args.get_single();
        let full_path = cur_dir.join(filename);
        if !full_path.exists() {
            eprintln!(
                "File does not exist: {}. Maybe there's a {}?\nInclude the extension {} as well.",
                full_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
                    .red(),
                "typo".to_string().yellow(),
                ".jpg/.JPG/.jpeg/.JPEG".to_string().green()
            );
        }
        match Compress::compress(
            full_path,
            cur_dir,
            self.args.get_quality(),
            Some("smoljpg_".to_string()),
        ) {
            Err(e) => eprintln!("{e}"),
            Ok(msg) => println!("{msg}"),
        }
        Ok(())
    }
}
