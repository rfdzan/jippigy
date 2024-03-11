use anyhow;
use std::env::args;
use std::env::{self, Args};
use std::io;
use std::path::{Path, PathBuf};
use turbojpeg::{compress_image, decompress_image, Subsamp::Sub2x2};
fn main() {
    let mut args = args();
    if args.len() != 3 {
        eprintln!(
            "Missing two required arguments in this exact order:\n1. Final quality\n2. Output directory name"
        );
        std::process::exit(1);
    }
    if let Err(e) = start(&mut args) {
        eprintln!("{e}");
    }
}
fn start(args: &mut Args) -> io::Result<()> {
    let quality = args.nth(1).unwrap_or_default().parse::<i32>().unwrap();
    let dir_name_from_args = args.nth(0).unwrap_or_default();
    let cur_dir = env::current_dir()?;
    let dir_name = Path::new(dir_name_from_args.as_str());
    if !cur_dir.join(dir_name).exists() {
        std::fs::create_dir(dir_name)?;
    }
    walk_dir(cur_dir, dir_name, quality)?;
    Ok(())
}
fn walk_dir(cur_dir: PathBuf, dir_name: &Path, quality: i32) -> io::Result<()> {
    for dent in std::fs::read_dir(cur_dir)? {
        let direntry = dent?;
        if direntry
            .path()
            .extension()
            .unwrap_or_default()
            .to_ascii_lowercase()
            == "jpg"
        {
            match compress(direntry.path(), &dir_name, quality) {
                Err(e) => {
                    eprintln!("{e}");
                }
                Ok(msg) => {
                    println!("{msg}");
                }
            };
        }
    }
    Ok(())
}
fn compress<T>(p: T, dir: &Path, q: i32) -> anyhow::Result<String>
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
        "done: {}",
        path_as_ref
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );
    Ok(success_msg)
}
