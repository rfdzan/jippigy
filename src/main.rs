use clap::Parser;
use smoljpg::{bulk::Parallel, single::Single};
use std::env::current_dir;
use std::path::PathBuf;
fn main() {
    let args = TaskArgs::parse();
    let cur_dir = current_dir().unwrap();
    if args.is_single() {
        Single::builder(args.get_single())
            .output_dir(args.get_output_dir())
            .with_quality(args.get_quality())
            .build()
            .do_single()
            .unwrap();
    } else {
        Parallel::builder(cur_dir.clone())
            .output_dir(args.get_output_dir())
            .unwrap()
            .with_quality(args.get_quality())
            .with_device(args.get_device())
            .build()
            .do_bulk()
            .unwrap();
    }
}
/// A multi-threaded JPG compression tool.
#[derive(Parser, Debug)]
struct TaskArgs {
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
    fn get_quality(&self) -> u8 {
        self.quality
    }
    /// Returns number of worker threads
    fn get_device(&self) -> u8 {
        self.device
    }
    /// Check if the task given is single image.
    fn is_single(&self) -> bool {
        if self.single.trim().is_empty() {
            return false;
        }
        true
    }
    /// Returns specified output dir.
    fn get_output_dir(&self) -> PathBuf {
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
    fn get_single(&self) -> PathBuf {
        let path = match current_dir() {
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
            Ok(p) => p,
        };
        path.join(self.single.as_str())
    }
}
