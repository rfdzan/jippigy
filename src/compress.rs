use anyhow::bail;
use colored::Colorize;
use image::EncodableLayout;
use img_parts::{jpeg::Jpeg, ImageEXIF, ImageICC};
use std::path::{Path, PathBuf};
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
pub(crate) struct Compress<T: AsRef<Path>> {
    path: PathBuf,
    output_dir: T,
    quality: u8,
    prefix: Option<String>,
}
impl<T: AsRef<Path>> Compress<T> {
    /// Creates a new compression task.
    pub(crate) fn new(path: PathBuf, output_dir: T, quality: u8, prefix: Option<String>) -> Self
    where
        T: AsRef<Path>,
    {
        Self {
            path,
            output_dir,
            quality: ValidQuality::from(quality).val(),
            prefix,
        }
    }
    /// Compresses the image with [turbojpeg](https://github.com/honzasp/rust-turbojpeg) while preserving exif data.
    pub(crate) fn compress(&self) -> Result<Vec<u8>, anyhow::Error> {
        let filename = self
            .path
            .clone()
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let with_exif_preserved = CompressImage::new(self.path.as_path(), self.quality)
            .compress()?
            .into_preserve_exif()
            .preserve_exif()?;
        // let before_size = with_exif_preserved.format_size_before();
        // let after_size = with_exif_preserved.format_size_after();
        // let name = {
        //     match self.prefix.clone() {
        //         None => filename,
        //         Some(n) => n + filename.as_str(),
        //     }
        // };
        let to_write = with_exif_preserved.get_compressed_bytes().map_err(|e| {
            eprintln!("{e}");
            e.context(format!("at: {}:{}:{}", file!(), line!(), column!()))
        });
        // std::fs::write(self.output_dir.as_ref().join(name.as_str()), to_write)?;
        // let success_msg = format!(
        //     "{name} before: {before_size} after: {after_size} ({}%)",
        //     self.quality
        // );
        to_write
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
    fn preserve_exif(self) -> anyhow::Result<Self> {
        let original_img_parts = Jpeg::from_bytes(self.original_bytes.clone().into())?;
        let exif = original_img_parts.exif().unwrap_or_default();
        let icc_profile = original_img_parts.icc_profile().unwrap_or_default();
        let mut compressed_img_part = Jpeg::from_bytes(self.compressed_bytes.into())?;
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
    fn get_compressed_bytes(self) -> Result<Vec<u8>, anyhow::Error> {
        if self.compressed_bytes.is_empty() && !self.with_exif_preserved.is_empty() {
            Ok(self.with_exif_preserved)
        } else {
            bail!("BUG: EXIF is not preserved.".red());
        }
    }
    /// Pretty formatting for original image size.
    fn format_size_before(&self) -> colored::ColoredString {
        let in_mbytes = (self.original_bytes.len()) as f64 / 1_000_000.0;
        let as_string = format!("{:.2} MB", in_mbytes);
        as_string.bright_red()
    }
    /// Pretty formatting for compressed image size.
    fn format_size_after(&self) -> colored::ColoredString {
        let in_mbytes = (self.with_exif_preserved.len()) as f64 / 1_000_000.0;
        let as_string = format!("{:.2} MB", in_mbytes);
        as_string.green()
    }
}
struct CompressImage<'a> {
    p: &'a Path,
    q: u8,
    original_bytes: Vec<u8>,
    compressed_bytes: Vec<u8>,
}
impl<'a> CompressImage<'a> {
    /// Creates a new image to be compressed.
    fn new(p: &'a Path, q: u8) -> Self {
        Self {
            p,
            q,
            original_bytes: Vec::new(),
            compressed_bytes: Vec::new(),
        }
    }
    /// Compresses image file, retaining original and compressed bytes. Returns self.
    fn compress(mut self) -> anyhow::Result<Self> {
        self.original_bytes = std::fs::read(self.p)?;
        let image: image::RgbImage = decompress_image(self.original_bytes.as_bytes())?;
        let jpeg_data = compress_image(&image, i32::from(self.q), Sub2x2)?;
        self.compressed_bytes = jpeg_data.as_bytes().to_owned();
        Ok(self)
    }
    /// Produce PreserveExif.
    fn into_preserve_exif(self) -> PreserveExif {
        PreserveExif {
            original_bytes: self.original_bytes,
            compressed_bytes: self.compressed_bytes,
            with_exif_preserved: Vec::new(),
        }
    }
}
