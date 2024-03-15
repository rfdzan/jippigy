use crate::TaskArgs;
use crossbeam::deque::Worker;
use std::env::current_dir;
use std::fs::DirEntry;
use std::io;
use std::path::Path;
use std::path::PathBuf;
/// Obtain tasks from the current working directory.
pub struct Tasks {
    queue: Worker<Option<DirEntry>>,
    device_num: u8,
    output_dir: PathBuf,
}
impl Tasks {
    /// Creates a new Task.
    pub fn create(args: &TaskArgs) -> io::Result<Self> {
        let cur_dir = current_dir()?;
        Ok(Self {
            queue: Tasks::get_tasks(&cur_dir)?,
            device_num: args.device,
            output_dir: Tasks::create_output_dir(&cur_dir, args.output_dir.as_str()),
        })
    }
    /// Returns a work-stealing queue from which worker threads are going to steal.
    pub fn get_main_worker(self) -> Worker<Option<DirEntry>> {
        self.queue
    }
    /// Returns the specified amount of worker threads to be used.
    pub fn get_device(&self) -> u8 {
        self.device_num - 1
    }
    /// Returns the output directory
    pub fn get_output_dir(&self) -> PathBuf {
        self.output_dir.clone()
    }
    /// Attempts to calculate the upper limit of the amount of work each thread should take.
    pub fn get_task_amount(&self) -> usize {
        {
            if self.device_num > 1 {
                let as_f64 = self.queue.len() as f64 / f64::from(self.device_num).ceil() + 1.0;
                as_f64 as usize
            } else {
                eprintln!("Minimum amount of device: 2");
                std::process::exit(1)
            }
        }
    }
    fn get_tasks(cur_dir: &PathBuf) -> io::Result<Worker<Option<DirEntry>>> {
        let read_dir = std::fs::read_dir(cur_dir)?;
        let worker = Worker::new_fifo();
        let _tasks = read_dir
            .map(|direntry| worker.push(direntry.ok()))
            .collect::<Vec<_>>();
        Ok(worker)
    }
    fn create_output_dir(cur_dir: &Path, output_dir: &str) -> PathBuf {
        let output_path = PathBuf::from(output_dir);
        if !cur_dir.join(output_path.as_path()).exists() {
            if let Err(e) = std::fs::create_dir(output_dir) {
                eprintln!("Cannot create output dir {output_dir}\n{e}")
            }
        }
        output_path
    }
}
