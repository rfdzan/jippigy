use colored::Colorize;
use image::EncodableLayout;
use img_parts::{jpeg::Jpeg, ImageEXIF, ImageICC};
use std::fs::DirEntry;
use std::io;
use std::path::Path;
use turbojpeg::{compress_image, decompress_image, Subsamp::Sub2x2};
/// Compression-related work.
pub struct Compress<T: AsRef<Path>> {
    direntry: Option<DirEntry>,
    output_dir: T,
    quality: u8,
}
impl<T: AsRef<Path>> Compress<T> {
    /// Creates a new compression task.
    pub fn new(direntry: Option<DirEntry>, output_dir: T, quality: u8) -> Self
    where
        T: AsRef<Path>,
    {
        Self {
            direntry,
            output_dir,
            quality,
        }
    }
    /// Start compression work.
    pub fn do_work(self) {
        let Some(val_direntry) = self.direntry else {
            return;
        };
        match Compress::compress(
            val_direntry.path(),
            self.output_dir.as_ref().into(),
            self.quality,
            None,
        ) {
            Err(e) => {
                eprintln!("{e}");
            }
            Ok(msg) => {
                println!("{msg}");
            }
        };
    }
    /// Compresses the image with [turbojpeg](https://github.com/honzasp/rust-turbojpeg) while preserving exif data.
    pub fn compress(
        p: T,
        output_dir: T,
        q: u8,
        custom_name: Option<String>,
    ) -> anyhow::Result<String>
    where
        T: AsRef<Path>,
    {
        if !output_dir.as_ref().exists() {
            eprintln!(
                "Output directory doesn't exist: {}",
                output_dir.as_ref().display()
            );
            std::process::exit(1);
        }
        let path_as_ref = p.as_ref();
        let filename = path_as_ref
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let with_exif_preserved = CompressImage::new(path_as_ref, q)
            .read()?
            .compress()?
            .preserve_exif()?;
        let before_size = with_exif_preserved.format_size_before();
        let after_size = with_exif_preserved.format_size_after();
        let name = {
            match custom_name {
                None => filename,
                Some(n) => n + filename.as_str(),
            }
        };
        std::fs::write(
            output_dir.as_ref().join(name.as_str()),
            with_exif_preserved.result().encoder().bytes(),
        )?;
        let success_msg = format!("{name} before: {before_size} after: {after_size}");
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
