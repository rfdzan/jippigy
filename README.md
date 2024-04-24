# jippigy
A multi-threaded JPEG compression crate, powered by [turbojpeg](https://github.com/honzasp/rust-turbojpeg).

This crate provides methods of compressing JPEG images in a single-threaded  or multi-threaded way. Both methods preserves [EXIF](https://en.wikipedia.org/wiki/Exif) data of the original JPEG through [img_parts](https://docs.rs/img-parts/latest/img_parts/) crate.

# 1.0.0 release!
Summary of breaking changes:
- This crate now deals only with your JPEG image bytes. Meaning you have to read your images into a `Vec<u8>` or `Vec<Vec<u8>>` where the former is for `Single` and the latter `Parallel`. Both outputs `Vec<u8>` image bytes which you can save or do more image processing with.
- It doesn't save images or create directories anymore, in fact it doesn't touch your filesystem at all.
- Compressing with `Parallel` now uses an iterator.

See the [CHANGELOG.md](https://github.com/rfdzan/jippigy/blob/master/CHANGELOG.md) for more details.
# Error building `turbojpeg`?
The problem is typically related to `turbojpeg-sys` (see this [question](https://github.com/rfdzan/smoljpg/issues/4#issuecomment-2036065574) and my [attempt](https://github.com/rfdzan/jippigy/actions/runs/8552014019/job/23432251063#step:3:327) at setting up CI for this crate).

To successfully build `turbojpeg-sys`, you need to install `cmake`, a C compiler (gcc, clang, etc.), and NASM in your system (See: [`turbojpeg`](https://github.com/honzasp/rust-turbojpeg)'s [requirements](https://github.com/honzasp/rust-turbojpeg?tab=readme-ov-file#requirements)). For more details, see [`turbojpeg-sys`](https://github.com/honzasp/rust-turbojpeg/tree/master/turbojpeg-sys)'s [`Building`](https://github.com/honzasp/rust-turbojpeg/tree/master/turbojpeg-sys#building) section.

 # Examples

 `with_` methods are optional.
 ## Single image compressions with `Single`
```rust
use jippigy::Single;
use image::{RgbImage, ImageFormat::Jpeg};
use std::io::Cursor;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vec: Vec<u8> = Vec::new();
    let img = RgbImage::new(1000, 1000);
    let _write = img.write_to(&mut Cursor::new(&mut vec), Jpeg)?;
    let _result: Vec<u8> = Single::from_bytes(vec)
        .with_quality(80)
        .build()
        .compress()?;
    Ok(())
}
```

 ## Multi-threaded bulk compressions with `Parallel`
 via `into_iter()`, `Parallel` converts into `ParallelIntoIterator` which returns items in the same order they were passed in. Which means, you can do something like the example below where you save the filenames of your JPEG into a vector, and later zip it with the `ParallelIntoIterator` you've made.
```rust
 use jippigy::Parallel;
 use std::path::PathBuf;
 use tempdir::TempDir;
 const TEST_DIR: &str = "./tests/images/";
 fn main() -> Result<(), Box<dyn std::error::Error>> {
     let image_dir_path = PathBuf::from(format!("{}", TEST_DIR));

     let mut vec_of_bytes = Vec::new();
     let mut list_of_names = Vec::new();

     // push the filenames and read bytes into a separate vector.
     for file in std::fs::read_dir(image_dir_path.clone())? {
         let filepath = file?.path();
         if filepath.is_file() {
             let filename = filepath.clone()
                 .file_name()
                 .and_then(|osstr| osstr.to_str())
                 .and_then(|a| Some(a.to_string()))
                 .unwrap_or_default();
             list_of_names.push(filename);
             let read_file = std::fs::read(filepath);
             vec_of_bytes.push(read_file?);
         }
     }
     // this temporary directory is here for doctest purposes,
     // but you will create your own directory.
     let tempdir = TempDir::new("compressed")?;

     // zip list_of_names vector with this iterator.
     for zipped in Parallel::from_vec(vec_of_bytes)
         .with_quality(50)
         .with_device(4)
         .build()
         .into_iter()
         .zip(list_of_names)
     {
         // saves compresssed JPEG with the original name.
         let (compressed_bytes, name) = zipped;
         if let Ok(bytes) = compressed_bytes {
             std::fs::write(
                 image_dir_path
                     .join(tempdir.path())
                     .join(format!("{name}").as_str()),
                 bytes,
             )?;
             println!("saved: {name}");
         }
     }
     tempdir.close()?;
     Ok(())
 }
```
