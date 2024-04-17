use crate::{error, Compress, DEVICE, QUALITY};
use crossbeam::channel;
use crossbeam::deque::{Steal, Stealer, Worker};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
/// Custom configuration for building a [`Parallel`].
/// This struct is not meant to be used directly.
/// Use [`Parallel::from_vec`] instead.
#[derive(Debug, Clone)]
pub struct ParallelBuilder {
    vec: Vec<Vec<u8>>,
    quality: u8,
    device_num: u8,
}
impl Default for ParallelBuilder {
    fn default() -> Self {
        Self {
            vec: Default::default(),
            quality: QUALITY,
            device_num: DEVICE,
        }
    }
}
impl ParallelBuilder {
    /// Builds a new [`Parallel`] with default or specified configuration.
    /// # Example
    /// This is the minimal requirements for using this method:
    /// ```
    /// # fn main() {
    /// # use jippigy::Parallel;
    /// let mut vector_of_bytes: Vec<Vec<u8>> = Vec::new();
    /// let _build = Parallel::from_vec(vector_of_bytes).build();
    /// # }
    /// ```
    pub fn build(self) -> Parallel {
        let (tx, rx) = channel::unbounded();
        Parallel {
            main_worker: Worker::new_fifo(),
            vec: self.vec,
            to_thread: StuffThatNeedsToBeSent {
                device_num: self.device_num,
                quality: self.quality,
                stealers: Vec::with_capacity(usize::from(self.device_num)),
            },
            transmitter: tx,
            receiver: rx,
        }
    }
    /// Specifies the quality of compressed images.
    /// Defaults to 95 (95% of the original quality).
    ///
    /// **This method is optional**.
    pub fn with_quality(self, quality: u8) -> ParallelBuilder {
        ParallelBuilder {
            vec: self.vec,
            quality,
            device_num: self.device_num,
        }
    }
    /// Specifies the number of threads to be used.
    /// Defaults to 2.
    ///
    /// **This method is optional**.
    /// # Warning
    /// Theoretically, using more threads would mean more workers working on your images.
    /// However, spawning many threads has diminishing returns and not to mention it can be costly.
    /// Experiment as you please, but if you don't know what number to put in simply don't use this method as it is optional.
    pub fn with_device(self, device_num: u8) -> ParallelBuilder {
        ParallelBuilder {
            vec: self.vec,
            quality: self.quality,
            device_num,
        }
    }
}
#[derive(Debug)]
pub struct StuffThatNeedsToBeSent {
    device_num: u8,
    quality: u8,
    stealers: Vec<Stealer<Vec<u8>>>,
}
impl StuffThatNeedsToBeSent {
    /// Compress images in parallel.
    fn send_to_threads(
        self,
        tx: channel::Sender<Result<Vec<u8>, error::Error>>,
    ) -> Vec<thread::JoinHandle<()>> {
        let mut handles = Vec::with_capacity(usize::from(self.device_num));
        let to_steal_from = Arc::new(Mutex::new(self.stealers));
        for _ in 0..self.device_num {
            let local_stealer = Arc::clone(&to_steal_from);
            let local_transmitter = tx.clone();
            let handle = thread::spawn(move || {
                let mut are_queues_empty = Vec::with_capacity(usize::from(self.device_num));
                let mut payload = Vec::with_capacity(1);
                loop {
                    {
                        let Some(stealer_guard) = local_stealer.try_lock().ok() else {
                            continue;
                        };
                        for stealer in stealer_guard.iter() {
                            let Steal::Success(jpeg_bytes) = stealer.steal() else {
                                continue;
                            };
                            payload.push(jpeg_bytes);
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
                        let compress_result = Compress::new(bytes, self.quality).compress();
                        match local_transmitter.send(compress_result) {
                            Err(e) => {
                                eprintln!("{e:#?}");
                            }
                            Ok(_) => {}
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
/// Parallelized compression task.
#[derive(Debug)]
pub struct Parallel {
    main_worker: Worker<Vec<u8>>,
    vec: Vec<Vec<u8>>,
    to_thread: StuffThatNeedsToBeSent,
    transmitter: channel::Sender<Result<Vec<u8>, error::Error>>,
    receiver: channel::Receiver<Result<Vec<u8>, error::Error>>,
}
impl Parallel {
    /// Creates a parallelized compression task from a vector of bytes. Returns a [`ParallelBuilder`].
    /// This method initializes the compression task with the following defaults:
    /// - Default final quality is 95% (95% of the original quality).
    /// - Default number of threads spawned is 2.
    /// # Example
    /// ```
    /// use jippigy::Parallel;
    /// fn main() {
    ///     let mut vector_of_bytes: Vec<Vec<u8>> = Vec::new();
    ///     let _parallel = Parallel::from_vec(vector_of_bytes);
    /// }
    /// ```
    /// In order to start the compression, it has to be built and made into an iterator with `into_iter`:
    /// ```
    /// use jippigy::Parallel;
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut vector_of_bytes: Vec<Vec<u8>> = Vec::new();
    ///     let into_iter = Parallel::from_vec(vector_of_bytes).build().into_iter();
    ///     for result in into_iter {
    ///         let bytes: Vec<u8> = result?;
    ///         // do something with the bytes.
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn from_vec(vec: Vec<Vec<u8>>) -> ParallelBuilder {
        ParallelBuilder {
            vec,
            quality: QUALITY,
            device_num: DEVICE,
        }
    }
    fn compress(mut self) -> Vec<JoinHandle<()>> {
        for _ in 0..self.to_thread.device_num {
            self.to_thread.stealers.push(self.main_worker.stealer());
        }
        for bytes in self.vec {
            self.main_worker.push(bytes);
        }
        let handles = self.to_thread.send_to_threads(self.transmitter);
        handles
    }
}
impl IntoIterator for Parallel {
    type Item = Result<Vec<u8>, error::Error>;
    type IntoIter = ParallelIntoIterator;
    fn into_iter(self) -> Self::IntoIter {
        let receiver = self.receiver.clone();
        let handles = self.compress();
        ParallelIntoIterator::new(receiver, handles)
    }
}

pub struct ParallelIntoIterator {
    recv: channel::Receiver<Result<Vec<u8>, error::Error>>,
}
impl ParallelIntoIterator {
    fn new(
        recv: channel::Receiver<Result<Vec<u8>, error::Error>>,
        _handles: Vec<JoinHandle<()>>,
    ) -> Self {
        Self { recv }
    }
}
impl Iterator for ParallelIntoIterator {
    type Item = Result<Vec<u8>, error::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(result) = self.recv.recv() {
            return Some(result);
        }
        None
    }
}
