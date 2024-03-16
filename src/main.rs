use clap::Parser;
use colored::Colorize;
use crossbeam::deque::Worker;
use smoljpg::{compress::Compress, task::Tasks, threads::TaskWorker, TaskArgs};
use std::io;
fn main() {
    let args = TaskArgs::parse();
    args.verify();
    if !args.is_single() {
        if let Err(e) = spawn_workers(args) {
            eprintln!("{e}");
        }
    } else {
        if let Err(e) = single(args) {
            eprintln!("{e}");
        }
    }
}
fn single(args: TaskArgs) -> io::Result<()> {
    let cur_dir = std::env::current_dir()?;
    let filename = args.get_single();
    let full_path = cur_dir.join(filename);
    if !full_path.exists() {
        eprintln!(
            "File does not exist: {}. Maybe there's a {}?\nInclude the extension {} as well.",
            full_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
                .red(),
            "typo".to_string().yellow(),
            ".jpg/.JPG/.jpeg/.JPEG".to_string().green()
        );
    }
    match Compress::compress(
        full_path,
        cur_dir,
        args.get_quality(),
        Some("smoljpg_".to_string()),
    ) {
        Err(e) => eprintln!("{e}"),
        Ok(msg) => println!("{msg}"),
    }
    Ok(())
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
        create_task.get_quality(),
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
