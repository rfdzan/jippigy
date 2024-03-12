use clap::Parser;
use crossbeam::deque::Steal;
use smoljpg::{Compress, TaskArgs, TaskWorker, Tasks};
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
    let main_worker = create_task.get_main_worker();
    let main_stealer = main_worker.stealer();
    let handles = TaskWorker::new(
        device_num,
        quality,
        dir_name.clone(),
        &main_stealer,
        task_amount,
    )
    .send_to_threads();
    // Makes sure all entries in the queue are consumed.
    while let Steal::Success(direntry) = main_stealer.steal() {
        Compress::new(direntry, dir_name.clone(), quality, 0).do_work();
    }
    for h in handles.into_iter() {
        h.join().unwrap();
    }
    Ok(())
}
