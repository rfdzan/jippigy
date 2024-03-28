use clap::Parser;
use smoljpg::{single::Single, threads::TaskWorker, TaskArgs};
use std::env::current_dir;
fn main() {
    let args = TaskArgs::parse();
    let cur_dir = current_dir().unwrap();
    args.verify();
    if args.is_single() {
        if let Err(e) = Single::new(
            args.get_single(),
            args.get_quality(),
            cur_dir,
            Some("smoljpg_".to_string()),
        )
        .do_single()
        {
            eprintln!("{e}");
        }
    } else {
        TaskWorker::builder(cur_dir.clone())
            .output_dir(cur_dir.join(args.get_output_dir()))
            .quality(args.get_quality())
            .device(args.get_device())
            .build()
            .do_bulk()
            .unwrap();
    }
}
