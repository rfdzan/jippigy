use crate::error;
use img_parts::{jpeg::Jpeg, ImageEXIF, ImageICC};
use turbojpeg::{compress_image, decompress_image, Subsamp::Sub2x2};

#[derive(Debug, Clone, Copy)]
struct ValidQuality(u8);
impl ValidQuality {
    fn val(&self) -> u8 {
        self.0
    }
}
impl From<u8> for ValidQuality {
    fn from(value: u8) -> Self {
        let val = {
            let max = std::cmp::min(value, 100);
            std::cmp::max(max, 1)
        };
        Self(val)
    }
}
/// Compression-related work.
#[derive(Debug, Clone)]
pub(crate) struct Compress {
    bytes: Vec<u8>,
    quality: u8,
}
impl Compress {
    /// Creates a new compression task.
    pub(crate) fn new(bytes: Vec<u8>, quality: u8) -> Self {
        Self {
            bytes,
            quality: ValidQuality::from(quality).val(),
        }
    }
    /// Compresses the image with [turbojpeg](https://github.com/honzasp/rust-turbojpeg) while preserving exif data.
    pub(crate) fn compress(&self) -> Result<Vec<u8>, error::Error> {
        let with_exif_preserved = CompressImage::new(self.bytes.clone(), self.quality)
            .compress()?
            .into_preserve_exif()
            .preserve_exif()?;
        // It must always be safe to unwrap this method.
        // Otherwise exif is not preserved.
        Ok(with_exif_preserved.get_compressed_bytes().unwrap())
    }
}
/// Compress an image, retaining its bytes before and after compression.
struct PreserveExif {
    original_bytes: Vec<u8>,
    compressed_bytes: Vec<u8>,
    with_exif_preserved: Vec<u8>,
}
impl PreserveExif {
    /// Using the bytes retained before and after compression,
    /// Parse EXIF information from the original bytes and write it
    /// into the compressed bytes.
    fn preserve_exif(self) -> Result<Self, error::Error> {
        let original_img_parts = match Jpeg::from_bytes(self.original_bytes.clone().into()) {
            Err(e) => return Err(error::Error::TurboJPEGError(e.to_string())),
            Ok(res) => res,
        };
        let exif = match original_img_parts.exif().ok_or(error::Error::ImgPartError(
            "Warning: No EXIF data found".to_string(),
        )) {
            Err(e) => {
                eprintln!("{e}");
                Vec::with_capacity(0).into()
            }
            Ok(res) => res,
        };
        let icc_profile = match original_img_parts
            .icc_profile()
            .ok_or(error::Error::ImgPartError(
                "Warning: No ICC profile found".to_string(),
            )) {
            Err(e) => {
                eprintln!("{e}");
                Vec::with_capacity(0).into()
            }
            Ok(res) => res,
        };
        let mut compressed_img_part = match Jpeg::from_bytes(self.compressed_bytes.into()) {
            Err(e) => return Err(error::Error::TurboJPEGError(e.to_string())),
            Ok(res) => res,
        };
        compressed_img_part.set_exif(exif.into());
        compressed_img_part.set_icc_profile(icc_profile.into());
        Ok(Self {
            original_bytes: self.original_bytes,
            compressed_bytes: Vec::with_capacity(0), // raw compressed bytes is no longer needed
            with_exif_preserved: compressed_img_part.encoder().bytes().to_vec(),
        })
    }
    /// Returns the compressed bytes with EXIF preserved.
    /// Fails if EXIF has not been preserved yet.
    fn get_compressed_bytes(self) -> Result<Vec<u8>, error::Error> {
        if self.compressed_bytes.is_empty() && !self.with_exif_preserved.is_empty() {
            Ok(self.with_exif_preserved)
        } else {
            Err(error::Error::JippigyError(
                "BUG: EXIF data is not preserved.".to_string(),
            ))
        }
    }
}
struct CompressImage {
    bytes: Vec<u8>,
    compressed_bytes: Vec<u8>,
    q: u8,
}
impl CompressImage {
    /// Creates a new image to be compressed.
    fn new(bytes: Vec<u8>, q: u8) -> Self {
        Self {
            q,
            bytes,
            compressed_bytes: Default::default(),
        }
    }
    /// Compresses image file, retaining original and compressed bytes. Returns self.
    fn compress(mut self) -> Result<Self, error::Error> {
        let image: image::RgbImage = match decompress_image(self.bytes.as_slice()) {
            Err(e) => return Err(error::Error::TurboJPEGError(e.to_string())),
            Ok(res) => res,
        };
        let jpeg_data = match compress_image(&image, i32::from(self.q), Sub2x2) {
            Err(e) => return Err(error::Error::TurboJPEGError(e.to_string())),
            Ok(res) => res,
        };
        self.compressed_bytes = jpeg_data.to_vec();
        Ok(self)
    }
    /// Produce PreserveExif.
    fn into_preserve_exif(self) -> PreserveExif {
        PreserveExif {
            original_bytes: self.bytes,
            compressed_bytes: self.compressed_bytes,
            with_exif_preserved: Vec::new(),
        }
    }
}
