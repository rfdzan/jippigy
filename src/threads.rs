use crate::compress::Compress;
use crossbeam::deque::{Steal, Stealer};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use std::{fs::DirEntry, path::PathBuf};
/// Worker threads.
pub struct TaskWorker {
    device_num: u8,
    quality: u8,
    dir_name: PathBuf,
    stealers: Vec<Stealer<Option<DirEntry>>>,
}
impl TaskWorker {
    /// Creates a new TaskWorker.
    pub fn new(
        device_num: u8,
        quality: u8,
        dir_name: PathBuf,
        stealers: Vec<Stealer<Option<DirEntry>>>,
    ) -> Self {
        Self {
            device_num,
            quality,
            dir_name,
            stealers,
        }
    }
    /// Distribute work among threads.
    /// This method consumes the TaskWorker and returns a vector containing the handles to each thread.
    pub fn send_to_threads(self) -> Vec<thread::JoinHandle<()>> {
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
