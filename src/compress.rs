use colored::Colorize;
use image::EncodableLayout;
use img_parts::{jpeg::Jpeg, ImageEXIF, ImageICC};
use std::io;
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
    pub(crate) fn compress(&self) -> anyhow::Result<String>
    where
        T: AsRef<Path>,
    {
        // TODO: make this output directory check into a bug check.
        if !self.output_dir.as_ref().exists() {
            eprintln!(
                "Output directory doesn't exist: {}",
                self.output_dir.as_ref().display()
            );
            std::process::exit(1);
        }
        let path_as_ref = self.path.clone();
        let filename = path_as_ref
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let with_exif_preserved = CompressImage::new(path_as_ref.as_path(), self.quality)
            .read()?
            .compress()?
            .preserve_exif()?;
        let before_size = with_exif_preserved.format_size_before();
        let after_size = with_exif_preserved.format_size_after();
        let name = {
            match self.prefix.clone() {
                None => filename,
                Some(n) => n + filename.as_str(),
            }
        };
        std::fs::write(
            self.output_dir.as_ref().join(name.as_str()),
            with_exif_preserved.result().encoder().bytes(),
        )?;
        let success_msg = format!(
            "{name} before: {before_size} after: {after_size} ({}%)",
            self.quality
        );
        Ok(success_msg)
    }
}
/// Compress an image, retaining its bytes before and after compression.
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
    /// Reads image file into Vec<u8>, returning Self.
    fn read(mut self) -> io::Result<Self> {
        self.original_bytes = std::fs::read(self.p)?;
        Ok(self)
    }
    /// Compress image file and retains the compressed bytes, returning Self.
    fn compress(mut self) -> anyhow::Result<Self> {
        let image: image::RgbImage = decompress_image(self.original_bytes.as_bytes())?;
        let jpeg_data = compress_image(&image, i32::from(self.q), Sub2x2)?;
        self.compressed_bytes = jpeg_data.as_bytes().to_owned();
        Ok(self)
    }
    /// Using the bytes retained before and after compression,
    /// Parse EXIF information from the original bytes and write it
    /// into the compressed bytes. Returns a `img_parts::jpeg::Jpeg`
    /// which we can convert later to a Byte.
    fn preserve_exif(self) -> anyhow::Result<CompressionResult> {
        let before_size = self.original_bytes.len();
        let after_size = self.compressed_bytes.len();
        let original_img_parts = Jpeg::from_bytes(self.original_bytes.into())?;
        let exif = original_img_parts.exif().unwrap_or_default();
        let icc_profile = original_img_parts.icc_profile().unwrap_or_default();
        let mut compressed_img_part = Jpeg::from_bytes(self.compressed_bytes.into())?;
        compressed_img_part.set_exif(exif.into());
        compressed_img_part.set_icc_profile(icc_profile.into());
        Ok(CompressionResult::store(
            compressed_img_part,
            before_size,
            after_size,
        ))
    }
}
/// Contains the result of compressed image
struct CompressionResult {
    compressed_img: img_parts::jpeg::Jpeg,
    before_length: usize,
    after_length: usize,
}
impl CompressionResult {
    /// Store the result of compressed image,
    /// along with additional information.
    fn store(
        compressed_img: img_parts::jpeg::Jpeg,
        before_length: usize,
        after_length: usize,
    ) -> Self {
        Self {
            compressed_img,
            before_length,
            after_length,
        }
    }
    /// Returns the compressed image as a `img_parts_jpeg::Jpeg`.
    fn result(self) -> img_parts::jpeg::Jpeg {
        self.compressed_img
    }
    /// Pretty formatting for original image size.
    fn format_size_before(&self) -> colored::ColoredString {
        let in_mbytes = (self.before_length) as f64 / 1_000_000.0;
        let as_string = format!("{:.2} MB", in_mbytes);
        as_string.bright_red()
    }
    /// Pretty formatting for compressed image size.
    fn format_size_after(&self) -> colored::ColoredString {
        let in_mbytes = (self.after_length) as f64 / 1_000_000.0;
        let as_string = format!("{:.2} MB", in_mbytes);
        as_string.green()
    }
}
