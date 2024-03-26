#![warn(missing_docs)]
//! A multi-threaded image compression tool, powered by [turbojpeg](https://github.com/honzasp/rust-turbojpeg).
use clap::Parser;
use std::{env, path::PathBuf};
/// Compression module.
pub mod compress;
/// Single-image tasks.
pub mod single;
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
    /// Returns number of worker threads
    pub fn get_device(&self) -> u8 {
        self.device
    }
    /// Check if the task given is single image.
    pub fn is_single(&self) -> bool {
        if self.single.trim().is_empty() {
            return false;
        }
        true
    }
    pub fn get_output_dir(&self) -> String {
        self.output_dir.clone()
    }
    /// Returns the single image path provided.
    pub fn get_single(&self) -> PathBuf {
        let path = match env::current_dir() {
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
            Ok(p) => p,
        };
        path.join(self.single.as_str())
    }
    /// Checks command-line input.
    pub fn verify(&self) {
        if self.quality < 1 || self.quality > 100 {
            eprintln!("Quality must be between 1 and 100");
            std::process::exit(1);
        }
    }
}
