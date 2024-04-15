
use crate::{error, Compress, QUALITY};
/// Creates a new Single struct for compressing single images.
#[derive(Debug, Clone)]
pub struct SingleBuilder {
    bytes_slice: Vec<u8>,
    quality: u8,
}
impl SingleBuilder {
    /// Specifies the quality of compressed images.
    /// Defaults to 95 (95% original quality).
    ///
    /// **This method is optional**.
    pub fn with_quality(self, quality: u8) -> SingleBuilder {
        SingleBuilder {
            bytes_slice: self.bytes_slice,
            quality,
        }
    }
}
impl SingleBuilder {
    /// Builds a new Single with custom configurations.
    /// # Example
    /// This is the minimum requirements for using this method:
    /// ```
    /// use jippigy::Single;
    /// fn main() {
    ///     let bytes: Vec<u8> = Vec::new();
    ///     let _build = Single::from_bytes(bytes).build();          
    /// }
    /// ```
    pub fn build(self) -> Single {
        Single {
            bytes_slice: self.bytes_slice,
            quality: self.quality,
        }
    }
}
/// Single image compressions.
#[derive(Debug, Clone)]
pub struct Single {
    bytes_slice: Vec<u8>,
    quality: u8,
}
impl Single {
    /// Creates a single image compression task from a given byte slice. Returns a [`SingleBuilder`].
    ///
    /// This method initializes the compression task with the following defaults:
    /// - Default final quality is 95% (95% of the original quality).
    /// # Example
    /// ```
    /// use jippigy::Single;     
    /// fn main() {
    ///     let bytes: Vec<u8> = Vec::new();   
    ///     let _single = Single::from_bytes(bytes);
    /// }
    /// ```
    pub fn from_bytes(bytes_slice: Vec<u8>) -> SingleBuilder {
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
    ///     let _write = img.write_to(&mut Cursor::new(&mut bytes), Jpeg)?;
    ///     let _result: Vec<u8> = Single::from_bytes(bytes)
    ///         .with_quality(80)
    ///         .build()
    ///         .compress()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn compress(self) -> Result<Vec<u8>, error::Error> {
        let compress = Compress::new(self.bytes_slice, self.quality).compress()?;
        Ok(compress)
    }
}
