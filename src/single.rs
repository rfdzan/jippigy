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
    ///
    /// **This method is optional**.
    pub fn with_quality(self, quality: u8) -> SingleBuilder<'a> {
        SingleBuilder {
            bytes_slice: self.bytes_slice,
            quality,
        }
    }
}
impl<'a> SingleBuilder<'a> {
    /// Builds a new Single with custom configurations.
    /// # Example
    /// This is the minimum requirements for using this method:
    /// ```
    /// use jippigy::Single;
    /// fn main() {
    ///     let bytes: Vec<u8> = Vec::new();
    ///     let _build = Single::from_bytes(bytes.as_slice()).build();          
    /// }
    /// ```
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
    /// Creates a single image compression task from a given byte slice. Returns a [`SingleBuilder`].
    ///
    /// This method initializes the compression task with the following defaults:
    /// - Default final quality is 95% (95% of the original quality).
    /// # Example
    /// ```
    /// use jippigy::Single;     
    /// fn main() {
    ///     let bytes: Vec<u8> = Vec::new();   
    ///     let _single = Single::from_bytes(bytes.as_slice());
    /// }
    /// ```
    pub fn from_bytes(bytes_slice: &'a [u8]) -> SingleBuilder {
        SingleBuilder {
            bytes_slice,
            quality: QUALITY,
        }
    }
    /// Compress a single image.
    /// # Example
    /// In order to start the compression, it has to be built first:
    /// ```
    /// use jippigy::Single;     
    /// fn main() {
    ///     let bytes: Vec<u8> = Vec::new();   
    ///     let _single = Single::from_bytes(bytes.as_slice()).build().compress();
    /// }
    /// ```
    pub fn compress(self) -> anyhow::Result<Vec<u8>> {
        Compress::new(self.bytes_slice.to_vec(), self.quality).compress()
    }
}
