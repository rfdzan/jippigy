use crate::compress::Compress;
use crate::TaskArgs;
use crossbeam::deque::Worker;
use crossbeam::deque::{Steal, Stealer};
use std::env::current_dir;
use std::fs::DirEntry;
use std::fs::ReadDir;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time;
/// Obtain tasks from the current working directory.
pub struct Tasks {
    queue: ReadDir,
    device_num: u8,
    quality: u8,
    output_dir: PathBuf,
}
impl Tasks {
    /// Creates a new Task.
    fn create(args: &TaskArgs) -> io::Result<Self> {
        let cur_dir = current_dir()?;
        Ok(Self {
            queue: Tasks::get_tasks(&cur_dir)?,
            device_num: args.device,
            quality: args.quality,
            output_dir: Tasks::create_output_dir(&cur_dir, args.output_dir.as_str()),
        })
    }
    /// Returns a work-stealing queue from which worker threads are going to steal.
    fn get_main_worker(self) -> ReadDir {
        self.queue
    }
    /// Returns the specified desired quality.
    fn get_quality(&self) -> u8 {
        self.quality
    }
    /// Returns the specified amount of worker threads to be used.
    fn get_device(&self) -> u8 {
        self.device_num
    }
    /// Returns the output directory
    fn get_output_dir(&self) -> PathBuf {
        self.output_dir.clone()
    }
    fn get_tasks(cur_dir: &PathBuf) -> io::Result<ReadDir> {
        std::fs::read_dir(cur_dir)
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
/// Worker threads.
pub struct TaskWorker {
    device_num: u8,
    quality: u8,
    dir_name: PathBuf,
    stealers: Vec<Stealer<Option<DirEntry>>>,
}
impl TaskWorker {
    /// Creates a new TaskWorker.
    pub fn new(device_num: u8, quality: u8, dir_name: PathBuf) -> Self {
        Self {
            device_num,
            quality,
            dir_name,
            stealers: Vec::with_capacity(usize::from(device_num)),
        }
    }
    /// Compress images in parallel.
    pub fn do_bulk(mut self, args: TaskArgs) -> io::Result<()> {
        let create_task = Tasks::create(&args)?;
        let device_num = create_task.get_device();
        let main_worker = Worker::new_fifo();
        for _ in 0..device_num {
            self.stealers.push(main_worker.stealer());
        }
        let handles = self.send_to_threads();
        for direntry in create_task.get_main_worker() {
            main_worker.push(direntry.ok());
        }
        for handle in handles.into_iter() {
            handle.join().unwrap();
        }
        Ok(())
    }
    /// Distribute work among threads.
    /// This method consumes the TaskWorker and returns a vector containing the handles to each thread.
    fn send_to_threads(self) -> Vec<thread::JoinHandle<()>> {
        let mut handles = Vec::with_capacity(usize::from(self.device_num));
        let to_steal_from = Arc::new(Mutex::new(self.stealers));
        for _ in 0..self.device_num {
            let local_stealer = Arc::clone(&to_steal_from);
            let thread_dir_name = self.dir_name.clone();
            let handle = thread::spawn(move || {
                let mut are_queues_empty = Vec::with_capacity(usize::from(self.device_num));
                let mut payload = Vec::with_capacity(1);
                // wait a bit for the main worker to have something in it.
                thread::sleep(time::Duration::from_millis(1));
                loop {
                    {
                        let Some(stealer_guard) = local_stealer.try_lock().ok() else {
                            continue;
                        };
                        for stealer in stealer_guard.iter() {
                            let Steal::Success(direntry) = stealer.steal() else {
                                continue;
                            };
                            payload.push(direntry);
                            break;
                        }
                        let _checks = stealer_guard
                            .iter()
                            .map(|stealer| {
                                if stealer.is_empty() {
                                    are_queues_empty.push(true);
                                } else {
                                    are_queues_empty.push(false);
                                }
                            })
                            .collect::<Vec<_>>();
                        // lock is no longer needed past this point
                    }
                    if let Some(direntry) = payload.pop() {
                        Compress::new(direntry, thread_dir_name.clone(), self.quality).do_work();
                    }
                    // if all stealers are empty, exit the loop.
                    if are_queues_empty.iter().all(|val| val == &true) {
                        break;
                    }
                    are_queues_empty.clear();
                    payload.clear();
                }
            });
            handles.push(handle);
        }
        handles
    }
}
