use crate::{compress::Compress, HasImageDir, HasOutputDir, DEVICE, QUALITY};
use crossbeam::deque::Worker;
use crossbeam::deque::{Steal, Stealer};
use std::fs::DirEntry;
use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

/// Custom configuration for building a Parallel.
#[derive(Debug, Clone)]
pub struct ParallelBuilder<IM, O, T> {
    image_dir: T,
    quality: u8,
    output_dir: T,
    device_num: u8,
    prefix: String,
    _marker: PhantomData<fn() -> (IM, O)>,
}
impl<IM, O, T> Default for ParallelBuilder<IM, O, T>
where
    T: AsRef<Path> + Default,
{
    fn default() -> Self {
        Self {
            image_dir: Default::default(),
            quality: QUALITY,
            output_dir: Default::default(),
            device_num: DEVICE,
            prefix: Default::default(),
            _marker: Default::default(),
        }
    }
}
impl<IM, O, T> ParallelBuilder<IM, O, T>
where
    T: AsRef<Path> + Default,
{
    /// Creates an output directory for compressed images.
    /// This method is required.
    pub fn output_dir(
        self,
        output_dir: T,
    ) -> io::Result<ParallelBuilder<HasImageDir, HasOutputDir, T>> {
        self.create_output_dir(&output_dir)?;
        Ok(ParallelBuilder {
            image_dir: self.image_dir,
            quality: self.quality,
            output_dir,
            device_num: self.device_num,
            prefix: self.prefix,
            _marker: PhantomData,
        })
    }
    /// Specifies the quality of compressed images.
    /// Defaults to 95 (95% of the original quality).
    pub fn with_quality(self, quality: u8) -> ParallelBuilder<HasImageDir, O, T> {
        ParallelBuilder {
            image_dir: self.image_dir,
            quality,
            device_num: self.device_num,
            output_dir: self.output_dir,
            prefix: self.prefix,
            _marker: PhantomData,
        }
    }
    /// Specifies a custom file name prefix for compressed images.
    pub fn with_prefix(self, prefix: String) -> ParallelBuilder<IM, O, T> {
        ParallelBuilder {
            image_dir: self.image_dir,
            quality: self.quality,
            device_num: self.device_num,
            output_dir: self.output_dir,
            prefix,
            _marker: PhantomData,
        }
    }
    /// Specifies the number of threads to be used.
    /// Defaults to 4.
    pub fn with_device(self, device_num: u8) -> ParallelBuilder<HasImageDir, O, T> {
        ParallelBuilder {
            image_dir: self.image_dir,
            quality: self.quality,
            output_dir: self.output_dir,
            device_num,
            prefix: self.prefix,
            _marker: PhantomData,
        }
    }
    /// Creates output directory. Exits if it fails.
    fn create_output_dir(&self, output_dir: &T) -> io::Result<()> {
        if !output_dir.as_ref().exists() {
            std::fs::create_dir(output_dir.as_ref())?
        }
        Ok(())
    }
}
impl<T> ParallelBuilder<HasImageDir, HasOutputDir, T>
where
    T: AsRef<Path>,
{
    /// Builds a new Parallel.
    pub fn build(self) -> Parallel {
        Parallel {
            device_num: self.device_num,
            quality: self.quality,
            image_dir: self.image_dir.as_ref().to_path_buf(),
            output_dir: self.output_dir.as_ref().to_path_buf(),
            stealers: Vec::with_capacity(usize::from(self.device_num)),
            prefix: self.prefix,
        }
    }
}
/// Worker threads.
#[derive(Debug, Clone)]
pub struct Parallel {
    device_num: u8,
    quality: u8,
    image_dir: PathBuf,
    output_dir: PathBuf,
    prefix: String,
    stealers: Vec<Stealer<Option<DirEntry>>>,
}
impl Parallel {
    /// Creates a new ParallelBuilder.
    pub fn builder<T: AsRef<Path> + Default>(image_dir: T) -> ParallelBuilder<HasImageDir, T, T> {
        ParallelBuilder {
            image_dir,
            quality: 50,
            output_dir: Default::default(),
            device_num: 4,
            prefix: Default::default(),
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
    /// This method consumes the Parallel and returns a vector containing the handles to each thread.
    fn send_to_threads(self) -> Vec<thread::JoinHandle<()>> {
        let mut handles = Vec::with_capacity(usize::from(self.device_num));
        let to_steal_from = Arc::new(Mutex::new(self.stealers));
        for _ in 0..self.device_num {
            let local_stealer = Arc::clone(&to_steal_from);
            let thread_output_dir = self.output_dir.clone();
            // let thread_custom_name = self.prefix.clone();
            let thread_custom_name = {
                if self.prefix.clone().trim().is_empty() {
                    None
                } else {
                    Some(self.prefix.clone())
                }
            };
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
                        if let Some(path) = direntry {
                            match Compress::new(
                                path.path(),
                                thread_output_dir.clone(),
                                self.quality,
                                thread_custom_name.clone(),
                            )
                            .compress()
                            {
                                Err(e) => {
                                    eprintln!("{e}");
                                }
                                Ok(msg) => {
                                    println!("{msg}");
                                }
                            }
                        }
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
