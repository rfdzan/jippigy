use crate::{create_output_dir, Compress, HasImageDir, HasOutputDir, DEVICE, QUALITY};
use crossbeam::channel;
use crossbeam::deque::Worker;
use crossbeam::deque::{Steal, Stealer};
use std::fs::DirEntry;
use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time;
/// Custom configuration for building a Parallel.
#[derive(Debug, Clone)]
pub struct ParallelBuilder<IM, O, T> {
    vec: Vec<Vec<u8>>,
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
            vec: Default::default(),
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
    // TODO: if we're going forward with returning images as bytes
    // `output_dir` will become obsolete and is not needed anymore.
    pub fn output_dir(
        self,
        output_dir: T,
    ) -> io::Result<ParallelBuilder<HasImageDir, HasOutputDir, T>> {
        create_output_dir(&output_dir)?;
        Ok(ParallelBuilder {
            vec: self.vec,
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
            vec: self.vec,
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
            vec: self.vec,
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
            vec: self.vec,
            quality: self.quality,
            output_dir: self.output_dir,
            device_num,
            prefix: self.prefix,
            _marker: PhantomData,
        }
    }
}
impl<T> ParallelBuilder<HasImageDir, HasOutputDir, T>
where
    T: AsRef<Path>,
{
    /// Builds a new Parallel.
    pub fn build(self) -> Parallel {
        let (tx, rx) = channel::unbounded();
        Parallel {
            device_num: self.device_num,
            quality: self.quality,
            vec: self.vec,
            output_dir: self.output_dir.as_ref().to_path_buf(),
            stealers: Vec::with_capacity(usize::from(self.device_num)),
            prefix: self.prefix,
            transmitter: tx,
            receiver: rx,
        }
    }
}
#[derive(Debug)]
pub struct StuffThatNeedsToBeSent {
    device_num: u8,
    quality: u8,
    vec: Vec<Vec<u8>>,
    output_dir: PathBuf,
    prefix: String,
    stealers: Vec<Stealer<Vec<u8>>>,
    transmitter: channel::Sender<Result<Vec<u8>, anyhow::Error>>,
    receiver: channel::Receiver<Result<Vec<u8>, anyhow::Error>>,
}
impl StuffThatNeedsToBeSent {
    fn send_to_threads(
        self,
        tx: channel::Sender<Result<Vec<u8>, anyhow::Error>>,
    ) -> Vec<thread::JoinHandle<()>> {
        let mut handles = Vec::with_capacity(usize::from(self.device_num));
        let to_steal_from = Arc::new(Mutex::new(self.stealers));
        for _ in 0..self.device_num {
            let local_stealer = Arc::clone(&to_steal_from);
            let local_transmitter = tx.clone();
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
                    if let Some(bytes) = payload.pop() {
                        let compress_result = Compress::new(
                            bytes,
                            thread_output_dir.clone(),
                            self.quality,
                            thread_custom_name.clone(),
                        )
                        .compress();
                        // TODO: return a struct containing original path + compression_result
                        local_transmitter.send(compress_result).unwrap();
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
/// Worker threads.
#[derive(Debug)]
pub struct Parallel {
    device_num: u8,
    quality: u8,
    vec: Vec<Vec<u8>>,
    output_dir: PathBuf,
    prefix: String,
    stealers: Vec<Stealer<Vec<u8>>>,
    transmitter: channel::Sender<Result<Vec<u8>, anyhow::Error>>,
    receiver: channel::Receiver<Result<Vec<u8>, anyhow::Error>>,
}
impl Parallel {
    /// Creates a new ParallelBuilder.
    pub fn from_vec<T: AsRef<Path> + Default>(
        vec: Vec<Vec<u8>>,
    ) -> ParallelBuilder<HasImageDir, T, T> {
        ParallelBuilder {
            vec,
            quality: QUALITY,
            output_dir: Default::default(),
            device_num: DEVICE,
            prefix: Default::default(),
            _marker: PhantomData,
        }
    }
    /// Compress images in parallel.
    fn compress(mut self) -> io::Result<Vec<JoinHandle<()>>> {
        let main_worker = Worker::new_fifo();
        let move_vec = self.vec;
        for _ in 0..self.device_num {
            self.stealers.push(main_worker.stealer());
        }
        let tx = self.transmitter.clone();
        for bytes in move_vec {
            main_worker.push(bytes);
        }
        let handles = self.send_to_threads(tx);
        Ok(handles)
    }
    /// Distribute work among threads. /// This method consumes the Parallel and returns a vector containing the handles to each thread.
    fn send_to_threads(
        self,
        tx: channel::Sender<Result<Vec<u8>, anyhow::Error>>,
    ) -> Vec<thread::JoinHandle<()>> {
        let mut handles = Vec::with_capacity(usize::from(self.device_num));
        let to_steal_from = Arc::new(Mutex::new(self.stealers));
        for _ in 0..self.device_num {
            let local_stealer = Arc::clone(&to_steal_from);
            let local_transmitter = tx.clone();
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
                    if let Some(bytes) = payload.pop() {
                        let compress_result = Compress::new(
                            bytes,
                            thread_output_dir.clone(),
                            self.quality,
                            thread_custom_name.clone(),
                        )
                        .compress();
                        // TODO: return a struct containing original path + compression_result
                        local_transmitter.send(compress_result).unwrap();
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
impl IntoIterator for Parallel {
    type Item = Result<Vec<u8>, anyhow::Error>;
    type IntoIter = ParallelIntoIterator;
    fn into_iter(self) -> Self::IntoIter {
        let receiver = self.receiver.clone();
        // TODO: this unwrap must be handled
        // not quite sure how to make into_iter() fallible.
        let handles = self.compress();
        ParallelIntoIterator::new(receiver, handles)
    }
}
pub struct ParallelIntoIterator {
    recv: channel::Receiver<Result<Vec<u8>, anyhow::Error>>,
}
impl ParallelIntoIterator {
    fn new(
        recv: channel::Receiver<Result<Vec<u8>, anyhow::Error>>,
        handles: Result<Vec<JoinHandle<()>>, io::Error>,
    ) -> Self {
        if let Err(e) = handles {
            eprintln!("{e}");
        }
        Self { recv }
    }
}
impl Iterator for ParallelIntoIterator {
    type Item = Result<Vec<u8>, anyhow::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Ok(result) = self.recv.recv() {
                return Some(result);
            } else {
                break;
            }
        }
        None
    }
}
