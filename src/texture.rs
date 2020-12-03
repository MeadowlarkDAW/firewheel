use crate::Point;
use image::{ImageBuffer, ImageError};
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

pub use crate::atlas::Handle;

pub(crate) enum Error {
    ImageError(ImageError, String),
    PixelBufferTooSmall(u32, u32),
}

/// Instructions for loading texture data.
pub struct Loader<'a> {
    dpi_mode: DpiMode,
    handle: &'a mut Handle,
}

impl<'a> Loader<'a> {
    /// Creates a new [`Loader`] using only a 1x dpi resolution source.
    /// The texture will be scaled up when using higher dpi resolutions.
    ///
    /// [`Loader`]: struct.Texture.html
    pub fn res_1x(handle: &'a mut Handle, source_1x: Source) -> Self {
        Self {
            dpi_mode: DpiMode::Only1x(source_1x),
            handle,
        }
    }

    /// Creates a new [`Loader`] with only a 2x hi-dpi resolution source.
    /// The texture will be scaled down when using lower dpi resolutions.
    ///
    /// [`Loader`]: struct.Texture.html
    pub fn res_2x(handle: &'a mut Handle, source_2x: Source) -> Self {
        Self {
            dpi_mode: DpiMode::Only2x(source_2x),
            handle,
        }
    }

    /// Creates a new [`Loader`] with both a 1x dpi and 2x hi-dpi resolution source.
    /// The application will decide which one to load based on the user's display.
    ///
    /// [`Loader`]: struct.Texture.html
    pub fn dual_res(
        handle: &'a mut Handle,
        source_1x: Source,
        source_2x: Source,
    ) -> Self {
        Self {
            dpi_mode: DpiMode::Both {
                source_1x,
                source_2x,
            },
            handle,
        }
    }

    pub(crate) fn handle(&mut self) -> &mut Handle {
        self.handle
    }

    pub(crate) fn load_bgra(
        &self,
        hi_dpi: bool,
    ) -> Result<(ImageBuffer<image::Bgra<u8>, Vec<u8>>, bool, Point), Error>
    {
        let (source, is_hi_dpi, center) = match &self.dpi_mode {
            DpiMode::Only1x(source) => (source, false, source.center()),
            DpiMode::Only2x(source) => (source, true, source.center()),
            DpiMode::Both {
                source_1x,
                source_2x,
            } => {
                if hi_dpi {
                    (source_2x, true, source_2x.center())
                } else {
                    (source_1x, false, source_1x.center())
                }
            }
        };

        Ok((source.load_bgra()?, is_hi_dpi, center))
    }
}

#[derive(Debug, Clone)]
enum DpiMode {
    Only1x(Source),
    Only2x(Source),
    Both {
        source_1x: Source,
        source_2x: Source,
    },
}

/// A [`Loader`] source.
///
/// [`Loader`]: struct.Texture`.html
#[derive(Debug, Clone)]
pub struct Source {
    data: Arc<Data>,
    center: Point,
}

impl Source {
    /// Creates a texture [`Source`] pointing to the image of the given path.
    ///
    /// Makes an educated guess about the image format by examining the data in the file.
    ///
    /// [`Source`]: struct.Source.html
    pub fn path<T: Into<PathBuf>>(path: T, center: Point) -> Source {
        Self::from_data(Data::Path(path.into()), center)
    }

    /// Creates a texture [`Source`] containing the image pixels directly. This
    /// function expects the input data to be provided as a `Vec<u8>` of BGRA
    /// pixels.
    ///
    /// This is useful if you have already decoded your image.
    ///
    /// [`Source`]: struct.Source.html
    pub fn pixels(
        width: u32,
        height: u32,
        pixels: Vec<u8>,
        center: Point,
    ) -> Source {
        Self::from_data(
            Data::Pixels {
                width,
                height,
                pixels,
            },
            center,
        )
    }

    /// Creates a texture [`Source`] containing the image data directly.
    ///
    /// Makes an educated guess about the image format by examining the given data.
    ///
    /// This is useful if you already have your image loaded in-memory, maybe
    /// because you downloaded or generated it procedurally.
    ///
    /// [`Source`]: struct.Source.html
    pub fn memory(bytes: Vec<u8>, center: Point) -> Source {
        Self::from_data(Data::Bytes(bytes), center)
    }

    fn from_data(data: Data, center: Point) -> Source {
        Source {
            data: Arc::new(data),
            center,
        }
    }

    /// Returns a reference to the texture [`Data`].
    ///
    /// [`Data`]: enum.Data.html
    pub fn data(&self) -> &Data {
        &self.data
    }

    /// Returns the origin of rotation of the texture.
    pub fn center(&self) -> Point {
        self.center
    }

    pub(crate) fn load_bgra(
        &self,
    ) -> Result<ImageBuffer<image::Bgra<u8>, Vec<u8>>, Error> {
        self.data.load_bgra()
    }
}

/// The data of a [`Loader`].
///
/// [`Loader`]: struct.Texture.html
#[derive(Clone)]
pub enum Data {
    /// File data
    Path(PathBuf),

    /// In-memory data
    Bytes(Vec<u8>),

    /// Decoded texture pixels in BGRA format.
    Pixels {
        /// The width of the texture.
        width: u32,
        /// The height of the texture.
        height: u32,
        /// The pixels.
        pixels: Vec<u8>,
    },
}

impl Data {
    pub(crate) fn load_bgra(
        &self,
    ) -> Result<ImageBuffer<image::Bgra<u8>, Vec<u8>>, Error> {
        match self {
            Data::Path(path) => {
                let img = match image::open(path) {
                    Ok(img) => img,
                    Err(e) => {
                        return Err(Error::ImageError(
                            e,
                            String::from(
                                path.to_str().unwrap_or("Invalid path"),
                            ),
                        ))
                    }
                };
                Ok(img.to_bgra8())
            }
            Data::Bytes(bytes) => {
                let img = match image::load_from_memory(bytes.as_slice()) {
                    Ok(img) => img,
                    Err(e) => {
                        return Err(Error::ImageError(e, String::from("")))
                    }
                };
                Ok(img.to_bgra8())
            }
            Data::Pixels {
                width,
                height,
                pixels,
            } => {
                if let Some(data) =
                    ImageBuffer::from_vec(*width, *height, pixels.to_vec())
                {
                    Ok(data)
                } else {
                    return Err(Error::PixelBufferTooSmall(*width, *height));
                }
            }
        }
    }
}

impl Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Path(path) => write!(f, "Path({:?})", path),
            Data::Bytes(_) => write!(f, "Bytes(...)"),
            Data::Pixels { width, height, .. } => {
                write!(f, "Pixels({} * {})", width, height)
            }
        }
    }
}
