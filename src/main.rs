use clap::Parser;
use crossbeam::deque::Worker;
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
    let main_worker = Worker::new_fifo();
    let mut stealers = Vec::with_capacity(usize::from(device_num));
    for _ in 0..device_num {
        stealers.push(main_worker.stealer());
    }
    let handles = TaskWorker::new(
        create_task.get_device(),
        args.get_quality(),
        create_task.get_output_dir(),
        stealers,
    )
    .send_to_threads();
    for direntry in create_task.get_main_worker() {
        main_worker.push(direntry.ok());
    }
    for handle in handles.into_iter() {
        handle.join().unwrap();
    }
    Ok(())
}
