use crate::compress::Compress;
use crossbeam::deque::{Steal, Stealer, Worker};
use std::sync::{Arc, Mutex};
use std::thread;
use std::{fs::DirEntry, path::PathBuf};
/// Worker threads.
pub struct TaskWorker<'a> {
    device_num: u8,
    quality: i32,
    dir_name: PathBuf,
    stealer: &'a Stealer<Option<DirEntry>>,
    task_amount: usize,
}
impl<'a> TaskWorker<'a> {
    /// Creates a new TaskWorker.
    pub fn new(
        device_num: u8,
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
    pub fn send_to_threads(self) -> Option<Vec<thread::JoinHandle<()>>> {
        let device_num_as_usize = usize::from(self.device_num);
        let mut handles = Vec::with_capacity(device_num_as_usize);
        let mut stealers = Vec::with_capacity(device_num_as_usize);
        let mut workers = Vec::with_capacity(device_num_as_usize);
        for _ in 0..self.device_num {
            let thread_worker = Worker::new_fifo();
            let _thread_stealer = self
                .stealer
                .steal_batch_with_limit(&thread_worker, self.task_amount);
            stealers.push(thread_worker.stealer());
            workers.push(thread_worker);
        }
        let to_steal_from = Arc::new(Mutex::new(stealers));
        for _ in 0..self.device_num {
            let thread_worker = workers.pop()?;
            let local_stealer = Arc::clone(&to_steal_from);
            let thread_dir_name = self.dir_name.clone();
            let handle = thread::spawn(move || {
                let mut queues_empty = Vec::with_capacity(device_num_as_usize);
                loop {
                    if let Some(direntry) = thread_worker.pop() {
                        Compress::new(direntry, thread_dir_name.clone(), self.quality).do_work();
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
                        Compress::new(direntry, thread_dir_name.clone(), self.quality).do_work();
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
        Some(handles)
    }
}
