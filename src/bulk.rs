use crate::{error, Compress, DEVICE, QUALITY};
use crossbeam::channel;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
/// Custom configuration for building a [`Parallel`].
/// This struct is not meant to be used directly.
/// Use [`Parallel::from_vec`] instead.
#[derive(Debug, Clone)]
pub struct ParallelBuilder {
    vec: VecDeque<(usize, Vec<u8>)>,
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
            to_thread: StuffThatNeedsToBeSent {
                vec: self.vec,
                device_num: self.device_num,
                quality: self.quality,
            },
            transmitter: tx,
            receiver: rx,
        }
    }
}
#[derive(Debug)]
pub struct StuffThatNeedsToBeSent {
    vec: VecDeque<(usize, Vec<u8>)>,
    device_num: u8,
    quality: u8,
}
impl StuffThatNeedsToBeSent {
    /// Compress images in parallel.
    fn send_to_threads(
        self,
        tx: channel::Sender<Result<Vec<u8>, error::Error>>,
    ) -> Vec<thread::JoinHandle<()>> {
        let mut handles = Vec::with_capacity(usize::from(self.device_num));
        let counter = Arc::new(Mutex::new(0usize));
        let to_steal_from = Arc::new(Mutex::new(self.vec));
        for _ in 0..self.device_num {
            let local_stealer = Arc::clone(&to_steal_from);
            let local_counter = Arc::clone(&counter);
            let local_transmitter = tx.clone();
            let handle = thread::spawn(move || {
                let mut payload = Vec::with_capacity(1);
                loop {
                    {
                        let Some(mut stealer_guard) = local_stealer.lock().ok() else {
                            continue;
                        };
                        if let Some(bytes) = stealer_guard.pop_front() {
                            payload.push(bytes);
                        } else {
                            break;
                        }
                        // lock is no longer needed past this point
                    }
                    if let Some(content) = payload.pop() {
                        let compress_result = Compress::new(content.1, self.quality).compress();
                        loop {
                            {
                                let Some(mut counter_guard) = local_counter.lock().ok() else {
                                    continue;
                                };
                                if !(*counter_guard == content.0) {
                                    continue;
                                } else {
                                    *counter_guard = *counter_guard + 1;
                                }
                                match local_transmitter.send(compress_result) {
                                    Err(e) => {
                                        eprintln!("{e:#?}");
                                    }
                                    Ok(_) => {}
                                }
                            }
                            break;
                        }
                    }
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
            vec: vec
                .into_iter()
                .enumerate()
                .map(|content| content)
                .collect::<VecDeque<(usize, Vec<u8>)>>(),
            quality: QUALITY,
            device_num: DEVICE,
        }
    }
    fn compress(self) -> Vec<JoinHandle<()>> {
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
