#![warn(missing_docs)]
//! A multi-threaded image compression tool, powered by [turbojpeg](https://github.com/honzasp/rust-turbojpeg).
use clap::Parser;
use crossbeam::deque::{Steal, Stealer, Worker};
use std::env::current_dir;
use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::{fs::DirEntry, path::PathBuf};
use turbojpeg::{compress_image, decompress_image, Subsamp::Sub2x2};
#[derive(Parser, Debug)]
/// Get arguments from the terminal.
pub struct TaskArgs {
    /// Ranges from 1 (smallest file, worst quality) to 100 (biggest file, best quality).
    #[arg(default_value_t = 50)]
    quality: u8,
    /// The output directory of compressed images.
    #[arg(default_value_t = format!("compressed"))]
    output_dir: String,
    /// The number of worker threads used.
    #[arg(short, default_value_t = 4)]
    device: u8,
}
impl TaskArgs {
    /// Returns the quality after compression.
    pub fn get_quality(&self) -> i32 {
        self.quality.into()
    }
    /// Checks command-line input.
    pub fn verify(&self) {
        if self.quality < 1 || self.quality > 100 {
            eprintln!("Quality must be between 1 and 100");
            std::process::exit(1);
        }
    }
}
/// Obtain tasks from the current working directory.
pub struct Tasks {
    queue: Worker<Option<DirEntry>>,
    device_num: u8,
    output_dir: PathBuf,
}
impl Tasks {
    /// Creates a new Task.
    pub fn create(args: &TaskArgs) -> io::Result<Tasks> {
        let cur_dir = current_dir()?;
        Ok(Tasks {
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
    pub fn get_device(&self) -> i32 {
        let device = self.device_num - 1;
        device.into()
    }
    /// Returns the output directory
    pub fn get_output_dir(&self) -> PathBuf {
        self.output_dir.clone()
    }
    /// Attempts to calculate the upper limit of the amount of work each thread should take.
    pub fn get_task_amount(&self) -> usize {
        let task_amount = {
            if self.device_num > 1 {
                let as_f64 =
                    self.queue.len() as f64 / f64::try_from(self.device_num).unwrap().ceil() + 1.0;
                as_f64 as usize
            } else {
                eprintln!("Minimum amount of device: 2");
                std::process::exit(1)
            }
        };
        task_amount
    }
    fn get_tasks(cur_dir: &PathBuf) -> io::Result<Worker<Option<DirEntry>>> {
        let read_dir = std::fs::read_dir(cur_dir)?;
        let worker = Worker::new_fifo();
        let _tasks = read_dir
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
/// Worker threads.
pub struct TaskWorker<'a> {
    device_num: i32,
    quality: i32,
    dir_name: PathBuf,
    stealer: &'a Stealer<Option<DirEntry>>,
    task_amount: usize,
}
impl<'a> TaskWorker<'a> {
    /// Creates a new TaskWorker.
    pub fn new(
        device_num: i32,
        quality: i32,
        dir_name: PathBuf,
        stealer: &'a Stealer<Option<DirEntry>>,
        task_amount: usize,
    ) -> Self {
        Self {
            device_num,
            quality,
            dir_name,
            stealer,
            task_amount,
        }
    }
    /// Distribute work among threads.
    /// This method consumes the TaskWorker and returns a vector containing the handles to each thread.
    pub fn send_to_threads(self) -> Vec<thread::JoinHandle<()>> {
        // self.device num is u8, so this conversion must always succeed.
        let device_num_as_usize =
            usize::try_from(self.device_num).expect("BUG: this conversion must always succeed");
        let mut handles = Vec::with_capacity(device_num_as_usize);
        let mut stealers = Vec::with_capacity(device_num_as_usize);
        let mut workers = Vec::with_capacity(device_num_as_usize);
        for _ in 0..self.device_num {
            let thread_worker = Worker::new_fifo();
            let _thread_stealer = self
                .stealer
                .steal_batch_with_limit(&thread_worker, self.task_amount);
            let _push_stealer = stealers.push(thread_worker.stealer());
            let _push_worker = workers.push(thread_worker);
        }
        let to_steal_from = Arc::new(Mutex::new(stealers));
        for id in 0..self.device_num {
            let thread_worker = workers.pop().unwrap();
            let local_stealer = Arc::clone(&to_steal_from);
            let thread_dir_name = self.dir_name.clone();
            let handle = thread::spawn(move || {
                let mut queues_empty = Vec::with_capacity(device_num_as_usize);
                loop {
                    if let Some(direntry) = thread_worker.pop() {
                        Compress::new(direntry, thread_dir_name.clone(), self.quality, id + 1)
                            .do_work();
                        continue;
                    }
                    let gain_lock = local_stealer.try_lock().ok();
                    let Some(list_of_stealers) = gain_lock else {
                        continue;
                    };
                    for stealer in list_of_stealers.iter() {
                        let Steal::Success(direntry) = stealer.steal() else {
                            continue;
                        };
                        Compress::new(direntry, thread_dir_name.clone(), self.quality, id + 1)
                            .do_work();
                        if stealer.is_empty() {
                            queues_empty.push(true);
                        } else {
                            queues_empty.push(false);
                        }
                    }
                    // If all worker threads have exhausted their queue,
                    // exit this loop
                    if queues_empty.iter().all(|val| val == &true) {
                        break;
                    }
                    queues_empty.clear();
                }
            });
            handles.push(handle);
        }
        return handles;
    }
}
/// Compression-related work.
pub struct Compress {
    direntry: Option<DirEntry>,
    dir_name: PathBuf,
    quality: i32,
    worker: i32,
}
impl Compress {
    /// Creates a new compression task.
    pub fn new(direntry: Option<DirEntry>, dir_name: PathBuf, quality: i32, worker: i32) -> Self {
        Self {
            direntry,
            dir_name,
            quality,
            worker,
        }
    }
    /// Compresses the image with [turbojpeg](https://github.com/honzasp/rust-turbojpeg).
    pub fn do_work(self) {
        let Some(val_direntry) = self.direntry else {
            return;
        };
        match Compress::compress(
            val_direntry.path(),
            self.dir_name,
            self.quality,
            self.worker + 1,
        ) {
            Err(e) => {
                eprintln!("{e}");
            }
            Ok(msg) => {
                println!("{msg}");
            }
        };
    }
    fn compress<T>(p: T, dir: PathBuf, q: i32, worker: i32) -> anyhow::Result<String>
    where
        T: AsRef<Path>,
    {
        let path_as_ref = p.as_ref();
        let filename = path_as_ref.file_name().unwrap_or_default();
        let read = std::fs::read(path_as_ref)?;
        let image: image::RgbImage = decompress_image(&read)?;
        let jpeg_data = compress_image(&image, q, Sub2x2)?;
        std::fs::write(dir.join(filename), jpeg_data)?;
        let success_msg = format!(
            "done: {} (worker {})",
            path_as_ref
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            worker
        );
        Ok(success_msg)
    }
}
