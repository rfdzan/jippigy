#![warn(missing_docs)]
//! A multi-threaded JPEG compression crate, powered by [turbojpeg](https://github.com/honzasp/rust-turbojpeg).
//!
//! This crate provides methods of compressing JPEG images in a single-threaded  or multi-threaded way. Both methods preserves [EXIF](https://en.wikipedia.org/wiki/Exif) data of the original JPEG through [img_parts](https://docs.rs/img-parts/latest/img_parts/) crate.
//!
//! [`Single`] is meant for single image compressions.
//! [`Parallel`] is meant for bulk compressions e.g. compressing a list of image bytes.
//!
//! As the name implies, [`Single`] is single-threaded whereas [`Parallel`] is multi-threaded.
//! # Error building `turbojpeg`?
//! The problem is typically related to `turbojpeg-sys` (see this [question](https://github.com/rfdzan/smoljpg/issues/4#issuecomment-2036065574) and my [attempt](https://github.com/rfdzan/jippigy/actions/runs/8552014019/job/23432251063#step:3:327) at setting up CI for this crate).
//!
//! To successfully build `turbojpeg-sys` you need to install `cmake`, a C compiler (gcc, clang, etc.), and NASM in your system (See: [`turbojpeg`]'s [requirements](https://github.com/honzasp/rust-turbojpeg?tab=readme-ov-file#requirements)). For more details, see [`turbojpeg-sys`]'s [`Building`] section.
//! # Examples
//!
//! `with_` methods are optional.

//! ## Single image compressions with [`Single`]
//!```
//! use jippigy::Single;
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use image::{RgbImage, ImageFormat::Jpeg};
//! # use std::io::Cursor;
//!     let mut vec: Vec<u8> = Vec::new();
//! # let img = RgbImage::new(1000, 1000);
//! # let _write = img.write_to(&mut Cursor::new(&mut vec), Jpeg)?;
//!     let _result: Vec<u8> = Single::from_bytes(vec)
//!         .with_quality(80)
//!         .build()
//!         .compress()?;
//!     Ok(())
//! }
//!```
//!
//! ## Multi-threaded bulk compressions with [`Parallel`]
//!```
//! use jippigy::Parallel;
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use image::{RgbImage, ImageFormat::Jpeg};
//! # use std::io::Cursor;
//!     let mut vector_of_bytes: Vec<Vec<u8>> = Vec::new();
//! # for _ in 0..10 {
//! #     let mut bytes = Vec::new();
//! #     let img = RgbImage::new(1000, 1000);
//! #     let _write = img.write_to(&mut Cursor::new(&mut bytes), Jpeg).unwrap();
//! #     vector_of_bytes.push(bytes);
//! # }
//!     for result in Parallel::from_vec(vector_of_bytes)
//!         .with_quality(80)
//!         .with_device(4) // how many threads to use.
//!         .build()
//!         .into_iter() {
//!         let compressed_bytes: Vec<u8> = result?;   
//!         // do something with the compressed results.
//!     }
//!     Ok(())
//! }
//!```
//!
//! [`Single`]: single::Single
//! [`Parallel`]: bulk::Parallel
//! [`turbojpeg`]: https://github.com/honzasp/rust-turbojpeg
//! [`turbojpeg-sys`]: https://github.com/honzasp/rust-turbojpeg/tree/master/turbojpeg-sys
//! [`Building`]: https://github.com/honzasp/rust-turbojpeg/tree/master/turbojpeg-sys#building
mod bulk;
mod compress;
mod defaults;
mod error;
mod single;

pub(crate) use self::compress::Compress;
pub(crate) use self::defaults::{DEVICE, QUALITY};
pub use self::{
    bulk::{Parallel, ParallelBuilder, ParallelIntoIterator},
    error::Error,
    single::{Single, SingleBuilder},
};
