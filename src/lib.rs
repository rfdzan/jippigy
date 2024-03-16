#![warn(missing_docs)]
//! A multi-threaded image compression tool, powered by [turbojpeg](https://github.com/honzasp/rust-turbojpeg).
use clap::Parser;
/// Compression module.
pub mod compress;
/// Obtaining work from parent directory.
pub mod task;
/// Parallelization module.
pub mod threads;
#[derive(Parser, Debug)]
/// Get arguments from the terminal.
pub struct TaskArgs {
    /// Ranges from 1 (smallest file, worst quality) to 100 (biggest file, best quality).
    #[arg(default_value_t = 50)]
    quality: u8,
    /// The output directory of compressed images.
    #[arg(default_value_t = format!("compressed"))]
    output_dir: String,
    #[arg(short, long, default_value_t = String::new())]
    single: String,
    /// The number of worker threads used.
    #[arg(short, default_value_t = 4)]
    device: u8,
}
impl TaskArgs {
    /// Returns the quality after compression.
    pub fn get_quality(&self) -> u8 {
        self.quality
    }
    pub fn is_single(&self) -> bool {
        if self.single.trim().is_empty() {
            return false;
        }
        true
    }
    pub fn get_single(&self) -> String {
        self.single.clone()
    }
    /// Checks command-line input.
    pub fn verify(&self) {
        if self.quality < 1 || self.quality > 100 {
            eprintln!("Quality must be between 1 and 100");
            std::process::exit(1);
        }
    }
}
