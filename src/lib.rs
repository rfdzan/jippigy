#![warn(missing_docs)]
//! A multi-threaded JPEG compression crate, powered by [turbojpeg](https://github.com/honzasp/rust-turbojpeg).
//!
//! This crate provides methods of compressing JPEG images in a single-threaded  or multi-threaded way. Both methods preserves [EXIF](https://en.wikipedia.org/wiki/Exif) data of the original JPEG through [img_parts](https://docs.rs/img-parts/latest/img_parts/) crate.
//!
//! [`Single`] is meant for single image compressions.
//! [`Parallel`] is meant for bulk compressions e.g. reading an entire directory and compressing any JPEG it finds.
//!
//! As the name implies, [`Single`] is single-threaded whereas [`Parallel`] is multi-threaded.
//! # Error building `turbojpeg`?
//! The problem is typically related to `turbojpeg-sys` (see this [question](https://github.com/rfdzan/smoljpg/issues/4#issuecomment-2036065574) and my [attempt](https://github.com/rfdzan/jippigy/actions/runs/8552014019/job/23432251063#step:3:327) at setting up CI for this crate).
//!
//!To successfully build `turbojpeg-sys`, you need to install `cmake`, a C compiler (gcc, clang, etc.), and NASM in your system (See: [`turbojpeg`]'s [requirements](https://github.com/honzasp/rust-turbojpeg?tab=readme-ov-file#requirements)). For more details, see [`turbojpeg-sys`]'s [`Building`] section.
//! # Examples
//! Both [`Single`] and [`Parallel`] require you to use both of their respective `output_dir()` methods (see: [`SingleBuilder.output_dir()`] and [`ParallelBuilder.output_dir()`] methods). `output_dir()` will attempt to create the directory if it doesn't exist. If it fails, it will return with an error before doing any expensive operations.
//!
//! `with_` methods are optional.

//! ## Single image compressions with [`Single`]
//!```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use image::RgbImage;
//! # use jippigy::single::Single;
//! # use tempdir;
//! # use std::fs;
//! # let temp_dir = tempdir::TempDir::new("example`").unwrap();
//! # let image_path = temp_dir.path().join("my_example_jpeg.jpg");
//! # fs::File::create(image_path.as_path()).unwrap();
//! # RgbImage::new(1000, 1000).save(image_path.as_path()).unwrap();
//! # let output_dir = temp_dir.into_path();
//! Single::builder(image_path)
//!     .output_dir(output_dir)? // This method is required.
//!     .with_quality(95)
//!     .with_prefix("my_prefix_".to_string())
//!     .build()
//!     .compress()?;
//! # Ok(())
//! # }
//!```
//!
//! ## Multi-threaded bulk compressions with [`Parallel`]
//!```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use image::RgbImage;
//! # use jippigy::bulk::Parallel;
//! # use tempdir;
//! # use std::fs;
//! # let temp_dir = tempdir::TempDir::new("example`").unwrap();
//! # let image_path = temp_dir.path().join("my_example_jpeg.jpg");
//! # fs::File::create(image_path.as_path()).unwrap();
//! # RgbImage::new(1000, 1000).save(image_path.as_path()).unwrap();
//! # let image_dir = temp_dir.into_path();
//! Parallel::builder(image_dir.clone())
//!     .output_dir(image_dir.join("compressed"))? // This method is required.
//!     .with_quality(95)
//!     .with_prefix("my_prefix_".to_string())
//!     .with_device(4) // Use 4 threads for this job.
//!     .build()
//!     .compress()?;
//! # Ok(())
//! # }
//!```
//! [`Single`]: single::Single
//! [`Parallel`]: bulk::Parallel
//! [`SingleBuilder.output_dir()`]: single::SingleBuilder#output_dir
//! [`ParallelBuilder.output_dir()`]: bulk::ParallelBuilder#output_dir
//! [`turbojpeg`]: https://github.com/honzasp/rust-turbojpeg
//! [`turbojpeg-sys`]: https://github.com/honzasp/rust-turbojpeg/tree/master/turbojpeg-sys
//! [`Building`]: https://github.com/honzasp/rust-turbojpeg/tree/master/turbojpeg-sys#building
/// Parallelization module.
pub mod bulk;
/// Compression module.
mod compress;
/// Default values.
mod defaults;
/// Single-image tasks.
pub mod single;
/// Type states of structs.
mod states;
use std::path::Path;

pub(crate) use self::compress::Compress;
pub(crate) use self::defaults::{DEVICE, QUALITY};
pub(crate) use self::states::{HasImage, HasImageDir, HasOutputDir};

/// Attempt to create an output directory.
pub(crate) fn create_output_dir(output_dir: &impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir(output_dir.as_ref()).or_else(|err| {
        if err.kind() == std::io::ErrorKind::AlreadyExists {
            Ok(())
        } else {
            Err(err)
        }
    })
}
