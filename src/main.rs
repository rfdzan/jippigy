use clap::Parser;
use smoljpg::{single::Single, threads::TaskWorker, TaskArgs};
use std::env::current_dir;
use std::io;
fn main() {
    let args = TaskArgs::parse();
    args.verify();
    if args.is_single() {
        if let Err(e) = Single::new(args).do_single() {
            eprintln!("{e}");
        }
    } else if let Err(e) = TaskWorker::new(
        args.get_device(),
        args.get_quality(),
        current_dir().unwrap(),
    )
    .do_bulk(args)
    {
        eprintln!("{e}");
    }
}
