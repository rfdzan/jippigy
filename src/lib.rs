#![warn(missing_docs)]
//! A multi-threaded image compression tool, powered by [turbojpeg](https://github.com/honzasp/rust-turbojpeg).
use clap::Parser;
use std::{env, path::PathBuf};
/// Compression module.
pub mod compress;
/// Default values.
pub mod defaults;
/// Single-image tasks.
pub mod single;
/// Parallelization module.
pub mod threads;
/// Current state has output directory.
pub enum HasOutputDir {}
#[derive(Parser, Debug)]
/// A multi-threaded JPG compression tool.
pub struct TaskArgs {
    /// Ranges from 1 (smallest file, worst quality) to 100 (biggest file, best quality).
    #[arg(default_value_t = 95)]
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
    /// Returns specified output dir.
    pub fn get_output_dir(&self) -> PathBuf {
        if !self.output_dir.is_empty() && self.is_single() {
            let output_to_path = PathBuf::from(self.output_dir.as_str());
            if !output_to_path.exists() {
                match std::fs::create_dir(output_to_path.as_path()) {
                    Err(e) => {
                        eprintln!("Error creating dir: {}\n{e}", output_to_path.display());
                        std::process::exit(1);
                    }
                    Ok(_) => {
                        return output_to_path;
                    }
                }
            }
        }
        PathBuf::from(self.output_dir.as_str())
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
    // / Checks command-line input.
    // pub fn verify(&self) {
    //     if self.quality < 1 || self.quality > 100 {
    //         eprintln!("Quality must be between 1 and 100");
    //         std::process::exit(1);
    //     }
    // }
}
