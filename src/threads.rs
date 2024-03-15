use crate::compress::Compress;
use crossbeam::deque::{Injector, Steal, Stealer, Worker};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use std::{fs::DirEntry, path::PathBuf};
/// Worker threads.
pub struct TaskWorker {
    device_num: u8,
    quality: i32,
    dir_name: PathBuf,
    stealers: Vec<Stealer<Option<DirEntry>>>,
    task_amount: usize,
    thread_stealers: Vec<Stealer<Option<DirEntry>>>,
    thread_workers: Vec<Worker<Option<DirEntry>>>,
}
impl TaskWorker {
    /// Creates a new TaskWorker.
    pub fn new(
        device_num: u8,
        quality: i32,
        dir_name: PathBuf,
        stealers: Vec<Stealer<Option<DirEntry>>>,
        task_amount: usize,
    ) -> Self {
        Self {
            device_num,
            quality,
            dir_name,
            stealers,
            task_amount,
            thread_stealers: Vec::new(),
            thread_workers: Vec::new(),
        }
    }
    /// TODO: doc
    // pub fn distribute_work(mut self) -> Self {
    //     for _ in 0..self.device_num {
    //         let thread_worker = Worker::new_fifo();
    //         let _thread_stealer = self
    //             .main_worker
    //             .stealer()
    //             .steal_batch_with_limit(&thread_worker, self.task_amount);
    //         self.thread_stealers.push(thread_worker.stealer());
    //         self.thread_workers.push(thread_worker);
    //     }
    //     self.thread_stealers.push(self.main_worker.stealer());
    //     self
    // }
    /// Distribute work among threads.
    /// This method consumes the TaskWorker and returns a vector containing the handles to each thread.
    pub fn send_to_threads(self) -> Option<Vec<thread::JoinHandle<()>>> {
        let mut handles = Vec::with_capacity(usize::from(self.device_num));
        let to_steal_from = Arc::new(Mutex::new(self.stealers));
        for _ in 0..self.device_num {
            // let thread_worker = self.thread_workers.pop()?;
            let local_stealer = Arc::clone(&to_steal_from);
            let thread_dir_name = self.dir_name.clone();
            let handle = thread::spawn(move || {
                thread::sleep(time::Duration::from_millis(50));
                let mut queues_empty = Vec::with_capacity(usize::from(self.device_num));
                loop {
                    println!("this runs");
                    // if let Steal::Success(direntry) = self.global_injector.steal() {
                    //     Compress::new(direntry, thread_dir_name.clone(), self.quality).do_work();
                    //     continue;
                    // }
                    // If the local worker is empty, steal work from other threads.
                    let gain_lock = local_stealer.try_lock().ok();
                    let Some(lock_list_of_stealers) = gain_lock else {
                        println!("contending for lock");
                        continue;
                    };
                    let mut temp_stealer = Vec::with_capacity(1);
                    for stealer in lock_list_of_stealers.iter() {
                        let Steal::Success(direntry) = stealer.steal() else {
                            continue;
                        };
                        temp_stealer.push(direntry);
                        break;
                    }
                    let _checks = lock_list_of_stealers
                        .iter()
                        .map(|stealer| {
                            if stealer.is_empty() {
                                queues_empty.push(true);
                            } else {
                                queues_empty.push(false);
                            }
                        })
                        .collect::<Vec<_>>();
                    // Release lock before doing compression.
                    drop(lock_list_of_stealers);
                    if let Some(direntry) = temp_stealer.pop() {
                        Compress::new(direntry, thread_dir_name.clone(), self.quality).do_work();
                    }
                    // If all worker threads have exhausted their queue,
                    // exit this loop
                    if queues_empty.iter().all(|val| val == &true) {
                        println!("a thread just exited");
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
