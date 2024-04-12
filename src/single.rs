use crate::{Compress, QUALITY};
/// Creates a new Single struct for compressing single images.
#[derive(Debug, Clone)]
pub struct SingleBuilder<'a> {
    bytes_slice: &'a [u8],
    quality: u8,
}
impl<'a> SingleBuilder<'a> {
    /// Specifies the quality of compressed images.
    /// Defaults to 95 (95% original quality).
    pub fn with_quality(self, quality: u8) -> SingleBuilder<'a> {
        SingleBuilder {
            bytes_slice: self.bytes_slice,
            quality,
        }
    }
}
impl<'a> SingleBuilder<'a> {
    /// Builds a new Single with custom configurations.
    pub fn build(self) -> Single<'a> {
        Single {
            bytes_slice: self.bytes_slice,
            quality: self.quality,
        }
    }
}
/// Single image compressions.
#[derive(Debug, Clone)]
pub struct Single<'a> {
    bytes_slice: &'a [u8],
    quality: u8,
}
impl<'a> Single<'a> {
    /// Creates a new Single configuration via SingleBuilder.
    pub fn from_bytes(bytes_slice: &'a [u8]) -> SingleBuilder {
        SingleBuilder {
            bytes_slice,
            quality: QUALITY,
        }
    }
    /// Compress a single image.
    pub fn compress(self) -> anyhow::Result<Vec<u8>> {
        Compress::new(self.bytes_slice.to_vec(), self.quality).compress()
    }
}
