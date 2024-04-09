use crate::{create_output_dir, Compress, HasImage, HasOutputDir, QUALITY};
use colored::Colorize;
use std::{
    io,
    marker::PhantomData,
    path::{Path, PathBuf},
};
/// Creates a new Single struct for compressing single images.
#[derive(Debug, Clone)]
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
    /// Output directory of image.
    /// This method is required.
    pub fn output_dir(self, output_dir: T) -> io::Result<SingleBuilder<IM, HasOutputDir, T>> {
        create_output_dir(&output_dir)?;
        Ok(SingleBuilder {
            image: self.image,
            quality: self.quality,
            output_dir,
            prefix: self.prefix,
            _marker: PhantomData,
        })
    }
    /// Specifies the quality of compressed images.
    /// Defaults to 95 (95% original quality).
    pub fn with_quality(self, quality: u8) -> SingleBuilder<IM, O, T> {
        SingleBuilder {
            image: self.image,
            quality,
            output_dir: self.output_dir,
            prefix: self.prefix,
            _marker: PhantomData,
        }
    }
    /// Specifies a custom file name prefix for compressed images.
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
    /// Builds a new Single with custom configurations.
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
#[derive(Debug, Clone)]
pub struct Single {
    image: PathBuf,
    quality: u8,
    output_dir: PathBuf,
    default_prefix: String,
}
impl Single {
    /// Creates a new Single configuration via SingleBuilder.
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
    pub fn compress(self) -> anyhow::Result<Vec<u8>> {
        self.exists()?.do_single()
    }
    /// Check whether or not image exists.
    fn exists(self) -> io::Result<Self> {
        std::fs::File::open(self.image.as_path())?;
        Ok(self)
    }
    /// Compress a single image.
    fn do_single(self) -> anyhow::Result<Vec<u8>> {
        let prefix = {
            if self.default_prefix.is_empty() {
                None
            } else {
                Some(self.default_prefix)
            }
        };
        Compress::new(self.image, self.output_dir, self.quality, prefix).compress()
    }
}
