#![warn(missing_docs)]
//! A multi-threaded image compression crate, powered by [turbojpeg](https://github.com/honzasp/rust-turbojpeg).
//!
//! This crate provides methods of compressing JPEG images in a single-threaded  or multi-threaded way. Both methods preserves [EXIF](https://en.wikipedia.org/wiki/Exif) data of the original JPEG through [img_parts](https://docs.rs/img-parts/latest/img_parts/) crate.
//!
//! [`Single`] is meant for single image compressions.
//! [`Parallel`] is meant for bulk compressions e.g. reading an entire directory and compressing any JPEG it finds.
//!
//! As the name implies, [`Single`] is single-threaded whereas [`Parallel`] is multi-threaded.
//! # Examples
//! Both [`Single`] and [`Parallel`] require you to use both of their respective `output_dir` methods. `with_` methods are optional.

//! ## Single image compressions with [`Single`]
//!```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use image::RgbImage;
//! # use smoljpg::single::Single;
//! # use tempdir;
//! # use std::fs;
//! # let temp_dir = tempdir::TempDir::new("example`").unwrap();
//! # let image_path = temp_dir.path().join("my_example_jpeg.jpg");
//! # fs::File::create(image_path.as_path()).unwrap();
//! # RgbImage::new(1000, 1000).save(image_path.as_path()).unwrap();
//! # let output_dir = temp_dir.into_path();
//! Single::builder(image_path)
//!     .output_dir(output_dir) // This method is required.
//!     .with_quality(95)
//!     .with_prefix("my_prefix_".to_string())
//!     .build()
//!     .do_single()?;
//! # Ok(())
//! # }
//!```
//!
//! ## Multi-threaded bulk compressions with [`Parallel`]
//! In this example, [`Parallel`] will attempt to create a separate directory `output_dir/compressed/` if it doesn't exist and save compressed images here.
//!```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use image::RgbImage;
//! # use smoljpg::bulk::Parallel;
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
//!     .do_bulk()?;
//! # Ok(())
//! # }
//!```
//! [`Single`]: single::Single
//! [`Parallel`]: bulk::Parallel

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
pub(crate) use self::compress::Compress;
pub(crate) use self::defaults::{DEVICE, QUALITY};
pub(crate) use self::states::{HasImage, HasImageDir, HasOutputDir};
