use anyhow;
use crossbeam::deque::Worker;
use std::env::args;
use std::env::{self, Args};
use std::fs::DirEntry;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use turbojpeg::{compress_image, decompress_image, Subsamp::Sub2x2};
fn main() {
    let mut args = args();
    if args.len() != 3 {
        eprintln!(
            "Missing two required arguments in this exact order:\n1. Final quality\n2. Output directory name"
        );
        std::process::exit(1);
    }
    if let Err(e) = get_params(&mut args) {
        eprintln!("{e}");
    }
}
fn get_params(args: &mut Args) -> io::Result<()> {
    let quality = args.nth(1).unwrap_or_default().parse::<i32>().unwrap();
    let dir_name_from_args = args.nth(0).unwrap_or_default();
    let cur_dir = env::current_dir()?;
    let dir_name = PathBuf::from(dir_name_from_args.as_str());
    if !cur_dir.join(dir_name.as_path()).exists() {
        std::fs::create_dir(dir_name.as_path())?;
    }
    spawn_workers(cur_dir, dir_name, quality)?;
    Ok(())
}
fn spawn_workers(cur_dir: PathBuf, dir_name: PathBuf, quality: i32) -> io::Result<()> {
    let main_worker = Worker::new_fifo();
    for dent in std::fs::read_dir(cur_dir)? {
        let direntry = dent?;
        main_worker.push(direntry);
    }
    let device_num = 4;
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
        println!("{}", thread_worker.len());
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
fn do_work(direntry: DirEntry, dir_name: PathBuf, quality: i32, worker: i32) {
    if direntry
        .path()
        .extension()
        .unwrap_or_default()
        .to_ascii_lowercase()
        == "jpg"
    {
        match compress(direntry.path(), dir_name, quality, worker) {
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
