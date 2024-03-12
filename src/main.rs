use anyhow;
use clap::Parser;
use crossbeam::deque::Worker;
use smoljpg::{TaskArgs, Tasks};
use std::fs::DirEntry;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use turbojpeg::{compress_image, decompress_image, Subsamp::Sub2x2};
fn main() {
    let args = TaskArgs::parse();
    if let Err(e) = spawn_workers(args) {
        eprintln!("{e}");
    }
}
fn spawn_workers(args: TaskArgs) -> io::Result<()> {
    let create_task = Tasks::create(&args)?;
    let device_num = create_task.get_device();
    let dir_name = create_task.get_output_dir();
    let main_worker = create_task.get_main_worker();
    let task_amount = {
        let as_f64 = main_worker.len() as f64 / f64::try_from(device_num).unwrap().ceil();
        as_f64 as usize
    };
    let main_stealer = main_worker.stealer();
    let clone_dir_name = dir_name.clone();
    let mut handles = vec![];
    for id in 0..device_num {
        let thread_dir_name = clone_dir_name.clone();
        let thread_worker = Worker::new_fifo();
        let _thread_stealer = main_stealer.steal_batch_with_limit(&thread_worker, task_amount);
        let quality = args.get_quality();
        let handle = thread::spawn(move || {
            while let Some(direntry) = thread_worker.pop() {
                do_work(direntry, thread_dir_name.clone(), quality, id);
            }
        });
        handles.push(handle);
    }
    for h in handles.into_iter() {
        h.join().unwrap();
    }
    Ok(())
}
fn do_work(direntry: Option<DirEntry>, dir_name: PathBuf, quality: i32, worker: i32) {
    let Some(val_direntry) = direntry else {
        return;
    };
    if val_direntry
        .path()
        .extension()
        .unwrap_or_default()
        .to_ascii_lowercase()
        == "jpg"
    {
        match compress(val_direntry.path(), dir_name, quality, worker) {
            Err(e) => {
                eprintln!("{e}");
            }
            Ok(msg) => {
                println!("{msg}");
            }
        };
    }
}
fn compress<T>(p: T, dir: PathBuf, q: i32, worker: i32) -> anyhow::Result<String>
where
    T: AsRef<Path>,
{
    let path_as_ref = p.as_ref();
    let filename = path_as_ref.file_name().unwrap_or_default();
    let read = std::fs::read(path_as_ref)?;
    let image: image::RgbImage = decompress_image(&read)?;
    let jpeg_data = compress_image(&image, q, Sub2x2)?;
    std::fs::write(dir.join(filename), jpeg_data)?;
    let success_msg = format!(
        "done: {} (worker {})",
        path_as_ref
            .file_name()
            .unwrap_or_default()
            .to_string_lossy(),
        worker
    );
    Ok(success_msg)
}
