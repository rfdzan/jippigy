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
```rust
use jippigy::Parallel;
use image::{RgbImage, ImageFormat::Jpeg};
use std::io::Cursor;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vector_of_bytes: Vec<Vec<u8>> = Vec::new();
    for _ in 0..10 {
        let mut bytes = Vec::new();
        let img = RgbImage::new(1000, 1000);
        let _write = img.write_to(&mut Cursor::new(&mut bytes), Jpeg).unwrap();
        vector_of_bytes.push(bytes);
    }
    for result in Parallel::from_vec(vector_of_bytes)
        .with_quality(80)
        .with_device(4) // how many threads to use.
        .build()
        .into_iter() {
        let compressed_bytes: Vec<u8> = result?;   
        // do something with the compressed results.
    }
    Ok(())
}
```
