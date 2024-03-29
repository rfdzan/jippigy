use crate::compress::Compress;
use crate::HasOutputDir;
use crossbeam::deque::Worker;
use crossbeam::deque::{Steal, Stealer};
use std::fs::DirEntry;
use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

/// Current state has image directory.
pub enum HasImageDir {}

/// Custom configuration for building a TaskWorker.
pub struct TaskWorkerBuilder<IM, O, T> {
    image_dir: T,
    quality: u8,
    output_dir: T,
    device_num: u8,
    _marker: PhantomData<fn() -> (IM, O)>,
}
impl<IM, O, T> Default for TaskWorkerBuilder<IM, O, T>
where
    T: AsRef<Path> + Default,
{
    fn default() -> Self {
        Self {
            image_dir: Default::default(),
            quality: 50,
            output_dir: Default::default(),
            device_num: 4,
            _marker: Default::default(),
        }
    }
}
impl<IM, O, T> TaskWorkerBuilder<IM, O, T>
where
    T: AsRef<Path> + Default,
{
    /// Creates an output directory for compressed images.
    /// This method is required.
    pub fn output_dir(self, output_dir: T) -> TaskWorkerBuilder<HasImageDir, HasOutputDir, T> {
        self.create_output_dir(&output_dir);
        TaskWorkerBuilder {
            image_dir: self.image_dir,
            quality: self.quality,
            output_dir,
            device_num: self.device_num,
            _marker: PhantomData,
        }
    }
    /// Specifies the quality of compressed images.
    /// Defaults to 50 (50% of the original quality).
    pub fn quality(self, quality: u8) -> TaskWorkerBuilder<HasImageDir, O, T> {
        TaskWorkerBuilder {
            image_dir: self.image_dir,
            quality,
            device_num: self.device_num,
            output_dir: self.output_dir,
            _marker: PhantomData,
        }
    }
    /// Specifies the number of threads to be used.
    /// Defaults to 4.
    pub fn device(self, device_num: u8) -> TaskWorkerBuilder<HasImageDir, O, T> {
        TaskWorkerBuilder {
            image_dir: self.image_dir,
            quality: self.quality,
            output_dir: self.output_dir,
            device_num,
            _marker: PhantomData,
        }
    }
    /// Creates output directory. Exits if it fails.
    fn create_output_dir(&self, output_dir: &T) {
        if !output_dir.as_ref().exists() {
            if let Err(e) = std::fs::create_dir(output_dir.as_ref()) {
                eprintln!(
                    "Cannot create output dir {}\n{e}",
                    self.output_dir.as_ref().display()
                );
                std::process::exit(1);
            }
        }
    }
}
impl<T> TaskWorkerBuilder<HasImageDir, HasOutputDir, T>
where
    T: AsRef<Path>,
{
    /// Builds a new TaskWorker.
    pub fn build(self) -> TaskWorker {
        TaskWorker {
            device_num: self.device_num,
            quality: self.quality,
            image_dir: self.image_dir.as_ref().to_path_buf(),
            output_dir: self.output_dir.as_ref().to_path_buf(),
            stealers: Vec::with_capacity(usize::from(self.device_num)),
        }
    }
}
/// Worker threads.
pub struct TaskWorker {
    device_num: u8,
    quality: u8,
    image_dir: PathBuf,
    output_dir: PathBuf,
    stealers: Vec<Stealer<Option<DirEntry>>>,
}
impl TaskWorker {
    /// Creates a new TaskWorkerBuilder.
    pub fn builder<T: AsRef<Path> + Default>(image_dir: T) -> TaskWorkerBuilder<HasImageDir, T, T> {
        TaskWorkerBuilder {
            image_dir,
            quality: 50,
            output_dir: Default::default(),
            device_num: 4,
            _marker: PhantomData,
        }
    }
    /// Compress images in parallel.
    pub fn do_bulk(mut self) -> io::Result<()> {
        let main_worker = Worker::new_fifo();
        for _ in 0..self.device_num {
            self.stealers.push(main_worker.stealer());
        }
        let read_dir = std::fs::read_dir(self.image_dir.as_path())?;
        let handles = self.send_to_threads();
        for direntry in read_dir {
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
            let thread_output_dir = self.output_dir.clone();
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
