use thiserror::Error;
#[non_exhaustive]
#[derive(Error, Debug, Clone)]
/// Errors emitted by jippigy.
pub enum Error {
    /// Represents critical errors. If you see it, please open an [issue](https://github.com/rfdzan/jippigy/issues).
    #[error("An internal error occured.")]
    JippigyInternalError(String),
    /// A compression error. See [turbojpeg](https://github.com/honzasp/rust-turbojpeg)'s error [enumerations](https://docs.rs/turbojpeg/latest/turbojpeg/enum.Error.html).
    #[error("TurboJPEGError:\n{0}")]
    TurboJPEGError(String),
    /// Error occured while attempting to read or write EXIF data and/or ICC profiles. See [img_part](https://github.com/paolobarbolini/img-parts)'s error [enumerations](https://docs.rs/img-parts/latest/img_parts/enum.Error.html).
    #[error("{0}")]
    ImgPartError(String),
}
