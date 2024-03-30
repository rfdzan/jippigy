use crate::{compress::Compress, defaults::*, HasOutputDir};
use colored::Colorize;
use std::{
    io,
    marker::PhantomData,
    path::{Path, PathBuf},
};
pub enum HasImage {}
pub struct SingleBuilder<IM, O, T> {
    image: T,
    quality: u8,
    output_dir: T,
    prefix: String,
    _marker: PhantomData<fn() -> (IM, O)>,
}
impl<IM, O, T> SingleBuilder<IM, O, T>
where
    T: AsRef<Path> + Default,
{
    pub fn output_dir(self, output_dir: T) -> SingleBuilder<IM, HasOutputDir, T> {
        SingleBuilder {
            image: self.image,
            quality: self.quality,
            output_dir,
            prefix: self.prefix,
            _marker: PhantomData,
        }
    }
    pub fn with_quality(self, quality: u8) -> SingleBuilder<IM, O, T> {
        SingleBuilder {
            image: self.image,
            quality,
            output_dir: self.output_dir,
            prefix: self.prefix,
            _marker: PhantomData,
        }
    }
    pub fn with_prefix(self, prefix: String) -> SingleBuilder<IM, O, T> {
        SingleBuilder {
            image: self.image,
            quality: self.quality,
            output_dir: self.output_dir,
            prefix,
            _marker: PhantomData,
        }
    }
}
impl<T> SingleBuilder<HasImage, HasOutputDir, T>
where
    T: AsRef<Path>,
{
    pub fn build(self) -> Single {
        Single {
            image: self.image.as_ref().to_path_buf(),
            quality: self.quality,
            output_dir: self.output_dir.as_ref().to_path_buf(),
            default_prefix: self.prefix,
        }
    }
}
/// Single image compressions.
pub struct Single {
    image: PathBuf,
    quality: u8,
    output_dir: PathBuf,
    default_prefix: String,
}
impl Single {
    pub fn builder<T: AsRef<Path> + Default>(image: T) -> SingleBuilder<HasImage, T, T> {
        SingleBuilder {
            image,
            quality: QUALITY,
            output_dir: Default::default(),
            prefix: Default::default(),
            _marker: PhantomData,
        }
    }
    /// Compress a single image.
    pub fn do_single(self) -> io::Result<()> {
        self.exists().compress();
        Ok(())
    }
    /// Check whether or not image exists.
    fn exists(self) -> Self {
        if !self.image.exists() {
            eprintln!(
                "File does not exist: {}. Maybe there's a {}?\nInclude the extension {} as well.",
                self.image
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
                    .red(),
                "typo".to_string().yellow(),
                ".jpg/.JPG/.jpeg/.JPEG".to_string().green()
            );
            std::process::exit(1);
        }
        self
    }
    /// Compress a single image.
    fn compress(self) {
        let prefix = {
            if self.default_prefix.is_empty() {
                None
            } else {
                Some(self.default_prefix)
            }
        };
        match Compress::new(self.image, self.output_dir, self.quality, prefix).compress() {
            Err(e) => eprintln!("{e}"),
            Ok(msg) => println!("{msg}"),
        }
    }
}
