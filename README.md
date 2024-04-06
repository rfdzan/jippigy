# jippigy
A multi-threaded image compression crate, powered by [turbojpeg](https://github.com/honzasp/rust-turbojpeg).

This crate provides methods of compressing JPEG images in a single-threaded  or multi-threaded way. Both methods preserves [EXIF](https://en.wikipedia.org/wiki/Exif) data of the original JPEG through [img_parts](https://docs.rs/img-parts/latest/img_parts/) crate.

# Error building `turbojpeg`?
The problem is typically related to `turbojpeg-sys` (see this [question](https://github.com/rfdzan/smoljpg/issues/4#issuecomment-2036065574) and my [attempt](https://github.com/rfdzan/jippigy/actions/runs/8552014019/job/23432251063#step:3:327) at setting up CI for this crate).

To successfully build `turbojpeg-sys`, you need to install `cmake`, a C compiler (gcc, clang, etc.), and NASM in your system (See: [`turbojpeg`](https://github.com/honzasp/rust-turbojpeg)'s [requirements](https://github.com/honzasp/rust-turbojpeg?tab=readme-ov-file#requirements)). For more details, see [`turbojpeg-sys`](https://github.com/honzasp/rust-turbojpeg/tree/master/turbojpeg-sys)'s [`Building`](https://github.com/honzasp/rust-turbojpeg/tree/master/turbojpeg-sys#building) section.

# Examples
Both `Single` and `Parallel` require you to use both of their respective `output_dir` methods. `with_` methods are optional.

## Single image compressions with `Single`
```rust
const IMAGE_DIR: &str = "/your/image/dir/";
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let image_dir_path = PathBuf::from(IMAGE_DIR);
    let _single = Single::builder(image_dir_path.join("DSC_0227.JPG"))
        .output_dir(image_dir_path.clone())? // This method is required.
        .with_prefix("jippigy_".to_string())
        .with_quality(95)
        .build()
        .compress()?;
    Ok(())
}
```
## Multi-threaded bulk compressions with `Parallel`
In this example, `Parallel` will attempt to create a separate directory `your/image/dir/compressed/` if it doesn't exist and save compressed images here.
```rust
const IMAGE_DIR: &str = "/your/image/dir/";
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _parallel = Parallel::builder(image_dir_path.clone())
        .output_dir(image_dir_path.join("compressed"))? // This method is required.
        .with_prefix("jippigy_".to_string())
        .with_quality(50)
        .with_device(6) // Use 6 threads for this job.
        .build()
        .compress()?;
    Ok(())
}
```
