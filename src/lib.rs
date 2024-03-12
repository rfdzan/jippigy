use clap::Parser;
use crossbeam::deque::Worker;
use std::env::current_dir;
use std::io;
use std::{fs::DirEntry, path::PathBuf};
#[derive(Parser, Debug)]
pub struct TaskArgs {
    /// Quality in ratio to the original.
    #[arg(default_value_t = 50)]
    quality: u8,
    /// The output directory of compressed images.
    #[arg(default_value_t = format!("compressed"))]
    output_dir: String,
    #[arg(short, default_value_t = 2)]
    device: u8,
}
impl TaskArgs {
    pub fn get_quality(&self) -> i32 {
        self.quality.into()
    }
}
pub struct Tasks {
    queue: Worker<Option<DirEntry>>,
    device_num: u8,
    output_dir: PathBuf,
}
impl Tasks {
    pub fn create(args: &TaskArgs) -> io::Result<Tasks> {
        let cur_dir = current_dir()?;
        Ok(Tasks {
            queue: Tasks::get_tasks(&cur_dir)?,
            device_num: args.device,
            output_dir: Tasks::create_output_dir(&cur_dir, args.output_dir.as_str()),
        })
    }
    pub fn get_main_worker(self) -> Worker<Option<DirEntry>> {
        self.queue
    }
    pub fn get_device(&self) -> i32 {
        self.device_num.into()
    }
    pub fn get_output_dir(&self) -> PathBuf {
        self.output_dir.clone()
    }
    fn get_tasks(cur_dir: &PathBuf) -> io::Result<Worker<Option<DirEntry>>> {
        let read_dir = std::fs::read_dir(cur_dir)?;
        let worker = Worker::new_fifo();
        let tasks = read_dir
            .map(|direntry| worker.push(direntry.ok()))
            .collect::<Vec<_>>();
        Ok(worker)
    }
    fn create_output_dir(cur_dir: &PathBuf, output_dir: &str) -> PathBuf {
        let output_path = PathBuf::from(output_dir);
        if !cur_dir.join(output_path.as_path()).exists() {
            std::fs::create_dir(output_dir).expect("Cannot create output dir {output_dir}\n");
        }
        output_path
    }
}
