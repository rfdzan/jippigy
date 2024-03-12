use clap::Parser;
use crossbeam::deque::{Stealer, Worker};
use std::env::current_dir;
use std::io;
use std::path::Path;
use std::thread;
use std::{fs::DirEntry, path::PathBuf};
use turbojpeg::{compress_image, decompress_image, Subsamp::Sub2x2};
#[derive(Parser, Debug)]
pub struct TaskArgs {
    /// Quality in ratio to the original.
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
        let device = self.device_num - 1;
        device.into()
    }
    pub fn get_output_dir(&self) -> PathBuf {
        self.output_dir.clone()
    }
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
pub struct TaskWorker<'a> {
    device_num: i32,
    quality: i32,
    dir_name: PathBuf,
    stealer: &'a Stealer<Option<DirEntry>>,
    task_amount: usize,
}
impl<'a> TaskWorker<'a> {
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
    pub fn send_to_threads(self) -> Vec<thread::JoinHandle<()>> {
        let mut handles = vec![];
        for id in 0..self.device_num {
            let thread_dir_name = self.dir_name.clone();
            let thread_worker = Worker::new_fifo();
            let _thread_stealer = self
                .stealer
                .steal_batch_with_limit(&thread_worker, self.task_amount);
            let handle = thread::spawn(move || {
                while let Some(direntry) = thread_worker.pop() {
                    Compress::new(direntry, thread_dir_name.clone(), self.quality, id + 1)
                        .do_work();
                }
            });
            handles.push(handle);
        }
        return handles;
    }
}
pub struct Compress {
    direntry: Option<DirEntry>,
    dir_name: PathBuf,
    quality: i32,
    worker: i32,
}
impl Compress {
    pub fn new(direntry: Option<DirEntry>, dir_name: PathBuf, quality: i32, worker: i32) -> Self {
        Self {
            direntry,
            dir_name,
            quality,
            worker,
        }
    }
    pub fn do_work(self) {
        let Some(val_direntry) = self.direntry else {
            return;
        };
        match Compress::compress(
            val_direntry.path(),
            self.dir_name,
            self.quality,
            self.worker,
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
