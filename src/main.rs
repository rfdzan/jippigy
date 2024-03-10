use std::env;
use std::env::args;
use std::path::Path;
use turbojpeg::{compress_image, decompress_image, Subsamp::Sub2x2};
fn main() {
    let mut args = args();
    if args.len() != 3 {
        eprintln!(
            "Missing two required arguments in this exact order:\n1. Final quality\n2. Output directory name"
        );
        std::process::exit(1);
    }
    let quality = args.nth(1).unwrap_or_default().parse::<i32>().unwrap();
    let dir_name_from_args = args.nth(0).unwrap_or_default();
    let cur_dir = env::current_dir().unwrap();
    let dir_name = Path::new(dir_name_from_args.as_str());
    if !cur_dir.join(dir_name).exists() {
        std::fs::create_dir(dir_name).unwrap();
    }
    for dent in std::fs::read_dir(cur_dir).unwrap() {
        let direntry = dent.unwrap();
        if direntry.path().is_file()
            && direntry
                .path()
                .extension()
                .unwrap_or_default()
                .to_ascii_lowercase()
                == "jpg"
        {
            compress(direntry.path(), &dir_name, quality);
            println!("done: {:?}", direntry.path().file_name());
        }
    }
}
fn compress<T>(p: T, dir: &Path, q: i32)
where
    T: AsRef<Path>,
{
    let path_as_ref = p.as_ref();
    let filename = path_as_ref.file_name().unwrap_or_default();
    let read = std::fs::read(path_as_ref).unwrap();
    let image: image::RgbImage = decompress_image(&read).unwrap();
    let jpeg_data = compress_image(&image, q, Sub2x2).unwrap();
    match std::fs::write(dir.join(filename), jpeg_data) {
        Err(e) => eprintln!("{e}"),
        Ok(_) => (),
    }
}
