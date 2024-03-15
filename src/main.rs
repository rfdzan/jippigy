use clap::Parser;
use crossbeam::deque::{Injector, Worker};
use smoljpg::{task::Tasks, threads::TaskWorker, TaskArgs};
use std::io;
fn main() {
    let args = TaskArgs::parse();
    args.verify();
    if let Err(e) = spawn_workers(args) {
        eprintln!("{e}");
    }
}
fn spawn_workers(args: TaskArgs) -> io::Result<()> {
    let create_task = Tasks::create(&args)?;
    let device_num = create_task.get_device();
    let dir_name = create_task.get_output_dir();
    let task_amount = create_task.get_task_amount();
    let quality = args.get_quality();
    let read_dir = create_task.get_main_worker();
    let main_worker = Worker::new_fifo();
    let mut stealers = Vec::with_capacity(usize::from(device_num));
    for _ in 0..device_num {
        stealers.push(main_worker.stealer());
    }
    let handles = TaskWorker::new(device_num, quality, dir_name.clone(), stealers, 60)
        /*.distribute_work()*/
        .send_to_threads();
    for direntry in read_dir {
        main_worker.push(direntry.ok())
    }
    println!("{}", main_worker.len());
    match handles {
        None => {
            eprintln!("BUG: number of workers pushed to and popped from is not the same.");
            std::process::exit(1);
        }
        Some(list_of_handles) => {
            for h in list_of_handles.into_iter() {
                if let Err(e) = h.join() {
                    eprintln!("{e:?}");
                }
            }
        }
    }
    Ok(())
}
