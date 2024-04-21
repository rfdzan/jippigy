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
//! ### New on 1.1.0:
//! Since update 1.1.0, [`Parallel`] returns items in the same order they were passed into, so you can do something like the example below where you save the filenames of your JPEG into a vector, and later zip it with the [`Parallel`] iterator you've made.
//!```
//! use jippigy::Parallel;
//! use std::path::PathBuf;
//! use tempdir::TempDir;
//! # const TEST_DIR: &str = "./tests/images/";
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let image_dir_path = PathBuf::from(format!("{}", TEST_DIR));
//!
//!     let mut vec_of_bytes = Vec::new();
//!     let mut list_of_names = Vec::new();
//!
//!     // push the filenames and read bytes into a vector each.
//!     for file in std::fs::read_dir(image_dir_path.clone())? {
//!         let filepath = file?.path();
//!         if filepath.is_file() {
//!             let filename = filepath.clone()
//!                 .file_name()
//!                 .and_then(|osstr| osstr.to_str())
//!                 .and_then(|a| Some(a.to_string()))
//!                 .unwrap_or_default();
//!             list_of_names.push(filename);
//!             let read_file = std::fs::read(filepath);
//!             vec_of_bytes.push(read_file?);
//!         }
//!     }

//!     // this temporary directory is here for doctest purposes,
//!     // but you will create your own directory.
//!     let tempdir = TempDir::new("compressed")?;
//!     for zipped in Parallel::from_vec(vec_of_bytes)
//!         .with_quality(50)
//!         .with_device(4)
//!         .build()
//!         .into_iter()
//!         .zip(list_of_names)
//!     {
//!         // saves compresssed JPEG with the original name.
//!         let (compressed_bytes, name) = zipped;
//!         if let Ok(bytes) = compressed_bytes {
//!             std::fs::write(
//!                 image_dir_path
//!                     .join(tempdir.path())
//!                     .join(format!("{name}").as_str()),
//!                 bytes,
//!             )?;
//!             println!("saved: {name}");
//!         }
//!     }
//!     tempdir.close()?;
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
    bulk::{Parallel, ParallelBuilder},
    error::Error,
    single::{Single, SingleBuilder},
};
