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
    /// use image::{RgbImage, ImageFormat::Jpeg};
    /// use std::io::Cursor;
    /// fn main() -> Result<(), Box<dyn std::error::Error>>{
    ///     let mut bytes = Vec::new();
    ///     let img = RgbImage::new(1000, 1000);
    ///     let _write = img.write_to(&mut Cursor::new(&mut bytes), Jpeg).unwrap();
    ///     let _result: Vec<u8> = Single::from_bytes(bytes.as_slice())
    ///         .with_quality(80)
    ///         .build()
    ///         .compress()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn compress(self) -> Result<Vec<u8>, anyhow::Error> {
        Ok(Compress::new(self.bytes_slice.to_vec(), self.quality).compress()?)
    }
}
