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
use std::thread;
use std::time;
/// Obtain tasks from the current working directory.
pub struct Tasks<T: AsRef<Path>> {
    queue: ReadDir,
    device_num: u8,
    quality: u8,
    output_dir: T,
}
impl<T: AsRef<Path>> Tasks<T> {
    /// Creates a new Task.
    fn create(queue: ReadDir, device_num: u8, quality: u8, output_dir: T) -> io::Result<Self>
    where
        T: AsRef<Path>,
    {
        Ok(Self {
            queue,
            device_num,
            quality,
            output_dir,
        })
    }
    /// Returns a work-stealing queue from which worker threads are going to steal.
    fn get_main_worker(self) -> ReadDir {
        self.queue
    }
    /// Returns the specified amount of worker threads to be used.
    fn get_device(&self) -> u8 {
        self.device_num
    }
    fn create_output_dir(&self) -> PathBuf {
        if !self.output_dir.as_ref().exists() {
            if let Err(e) = std::fs::create_dir(self.output_dir.as_ref()) {
                eprintln!(
                    "Cannot create output dir {}\n{e}",
                    self.output_dir
                        .as_ref()
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                )
            }
        }
        self.output_dir.as_ref().to_path_buf()
    }
}
/// Worker threads.
pub struct TaskWorker<T> {
    device_num: u8,
    quality: u8,
    image_dir: T,
    output_dir: T,
    stealers: Vec<Stealer<Option<DirEntry>>>,
}
impl<T: AsRef<Path> + 'static> TaskWorker<T> {
    /// Creates a new TaskWorker.
    pub fn new(image_dir: T, output_dir: T, device_num: u8, quality: u8) -> Self
    where
        T: AsRef<Path> + 'static,
    {
        Self {
            image_dir,
            output_dir,
            device_num,
            quality,
            stealers: Vec::with_capacity(usize::from(device_num)),
        }
    }
    /// Compress images in parallel.
    pub fn do_bulk(mut self) -> io::Result<()> {
        let create_task = Tasks::create(
            std::fs::read_dir(self.image_dir.as_ref()).unwrap(),
            self.device_num,
            self.quality,
            self.output_dir.as_ref().to_path_buf().clone(),
        )?;
        create_task.create_output_dir();
        let main_worker = Worker::new_fifo();
        for _ in 0..self.device_num {
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
            let thread_output_dir = self.output_dir.as_ref().to_path_buf().clone();
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
                        Compress::new(direntry, thread_output_dir.clone(), self.quality).do_work();
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
