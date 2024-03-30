use clap::Parser;
use smoljpg::{single::Single, threads::TaskWorker, TaskArgs};
use std::env::current_dir;
fn main() {
    let args = TaskArgs::parse();
    let cur_dir = current_dir().unwrap();
    if args.is_single() {
        Single::builder(args.get_single())
            .output_dir(args.get_output_dir())
            .with_quality(args.get_quality())
            .build()
            .do_single()
            .unwrap();
    } else {
        TaskWorker::builder(cur_dir.clone())
            .output_dir(args.get_output_dir())
            .unwrap()
            .with_quality(args.get_quality())
            .with_device(args.get_device())
            .build()
            .do_bulk()
            .unwrap();
    }
}
