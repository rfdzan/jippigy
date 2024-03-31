#![warn(missing_docs)]
//! A multi-threaded image compression tool, powered by [turbojpeg](https://github.com/honzasp/rust-turbojpeg).
/// Parallelization module.
pub mod bulk;
/// Compression module.
pub mod compress;
/// Default values.
mod defaults;
/// Single-image tasks.
pub mod single;
/// Type states of structs.
mod states;
pub(crate) use self::defaults::{DEVICE, QUALITY};
pub(crate) use self::states::{HasImage, HasImageDir, HasOutputDir};
