use thiserror::Error;
#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("A jippigy error occured.")]
    JippigyError(String),
    #[error("A compression error occured.\n{0}")]
    TurboJPEGError(String),
    #[error("{0}")]
    ImgPartError(String),
}
