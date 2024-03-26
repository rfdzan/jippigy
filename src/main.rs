use clap::Parser;
use smoljpg::{single::Single, threads::TaskWorker, TaskArgs};
use std::env::current_dir;
fn main() {
    let args = TaskArgs::parse();
    args.verify();
    if args.is_single() {
        if let Err(e) = Single::new(
            args.get_single(),
            args.get_quality(),
            current_dir().unwrap(),
            Some("smoljpg_".to_string()),
        )
        .do_single()
        {
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
