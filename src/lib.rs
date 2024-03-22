#![warn(missing_docs)]
//! A multi-threaded image compression tool, powered by [turbojpeg](https://github.com/honzasp/rust-turbojpeg).
use clap::Parser;
/// Compression module.
pub mod compress;
/// Single-image tasks.
pub mod single;
/// Obtaining work from parent directory.
pub mod task;
/// Parallelization module.
pub mod threads;
#[derive(Parser, Debug)]
/// A multi-threaded JPG compression tool.
pub struct TaskArgs {
    /// Ranges from 1 (smallest file, worst quality) to 100 (biggest file, best quality).
    #[arg(default_value_t = 50)]
    quality: u8,
    /// The output directory of compressed images.
    #[arg(short, default_value_t = format!("compressed"))]
    output_dir: String,
    /// Single image compression.
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
    /// Check if the task given is single image.
    pub fn is_single(&self) -> bool {
        if self.single.trim().is_empty() {
            return false;
        }
        true
    }
    /// Returns the single image file name provided.
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
