use crate::{Hasher, Point};
use image::{ImageBuffer, ImageError};
use std::fmt::Debug;
use std::hash::{Hash, Hasher as _};
use std::path::PathBuf;
use std::sync::Arc;

pub(crate) enum TextureError {
    ImageError(ImageError, String),
    PixelBufferTooSmall(u32, u32),
}

/// Texture data.
#[derive(Debug, Clone)]
pub struct Texture {
    dpi_mode: DpiMode,
}

impl Texture {
    /// Creates a new [`Texture`] using only a 1x dpi resolution source.
    /// The texture will be scaled up when using higher dpi resolutions.
    ///
    /// [`Texture`]: struct.Texture.html
    pub fn res_1x(source_1x: TextureSource) -> Self {
        Self {
            dpi_mode: DpiMode::Only1x(source_1x),
        }
    }

    /// Creates a new [`Texture`] with only a 2x hi-dpi resolution source.
    /// The texture will be scaled down when using lower dpi resolutions.
    ///
    /// [`Texture`]: struct.Texture.html
    pub fn res_2x(source_2x: TextureSource) -> Self {
        Self {
            dpi_mode: DpiMode::Only2x(source_2x),
        }
    }

    /// Creates a new [`Texture`] with both a 1x dpi and 2x hi-dpi resolution source.
    /// The application will decide which one to load based on the user's display.
    ///
    /// [`Texture`]: struct.Texture.html
    pub fn dual_res(
        source_1x: TextureSource,
        source_2x: TextureSource,
    ) -> Self {
        Self {
            dpi_mode: DpiMode::Both {
                source_1x,
                source_2x,
            },
        }
    }

    pub(crate) fn load_bgra(
        &self,
        hi_dpi: bool,
    ) -> Result<
        (ImageBuffer<image::Bgra<u8>, Vec<u8>>, bool, Point),
        TextureError,
    > {
        let (source, is_hi_dpi, rotation_origin) = match &self.dpi_mode {
            DpiMode::Only1x(source) => {
                (source, false, source.rotation_origin())
            }
            DpiMode::Only2x(source) => (source, true, source.rotation_origin()),
            DpiMode::Both {
                source_1x,
                source_2x,
            } => {
                if hi_dpi {
                    (source_2x, true, source_2x.rotation_origin())
                } else {
                    (source_1x, false, source_1x.rotation_origin())
                }
            }
        };

        Ok((source.load_bgra()?, is_hi_dpi, rotation_origin))
    }
}

#[derive(Debug, Clone)]
enum DpiMode {
    Only1x(TextureSource),
    Only2x(TextureSource),
    Both {
        source_1x: TextureSource,
        source_2x: TextureSource,
    },
}

/// A [`Texture`] source.
///
/// [`Texture`]: struct.Texture`.html
#[derive(Debug, Clone)]
pub struct TextureSource {
    data: Arc<Data>,
    rotation_origin: Point,
}

impl TextureSource {
    /// Creates a texture [`TextureSource`] pointing to the image of the given path.
    ///
    /// Makes an educated guess about the image format by examining the data in the file.
    ///
    /// [`TextureSource`]: struct.TextureSource.html
    pub fn path<T: Into<PathBuf>>(
        path: T,
        rotation_origin: Option<Point>,
    ) -> TextureSource {
        Self::from_data(Data::Path(path.into()), rotation_origin)
    }

    /// Creates a texture [`TextureSource`] containing the image pixels directly. This
    /// function expects the input data to be provided as a `Vec<u8>` of BGRA
    /// pixels.
    ///
    /// This is useful if you have already decoded your image.
    ///
    /// [`TextureSource`]: struct.TextureSource.html
    pub fn pixels(
        width: u32,
        height: u32,
        pixels: Vec<u8>,
        rotation_origin: Option<Point>,
    ) -> TextureSource {
        Self::from_data(
            Data::Pixels {
                width,
                height,
                pixels,
            },
            rotation_origin,
        )
    }

    /// Creates a texture [`TextureSource`] containing the image data directly.
    ///
    /// Makes an educated guess about the image format by examining the given data.
    ///
    /// This is useful if you already have your image loaded in-memory, maybe
    /// because you downloaded or generated it procedurally.
    ///
    /// [`TextureSource`]: struct.TextureSource.html
    pub fn memory(
        bytes: Vec<u8>,
        rotation_origin: Option<Point>,
    ) -> TextureSource {
        Self::from_data(Data::Bytes(bytes), rotation_origin)
    }

    fn from_data(data: Data, rotation_origin: Option<Point>) -> TextureSource {
        TextureSource {
            data: Arc::new(data),
            rotation_origin: rotation_origin.unwrap_or(Point::ORIGIN),
        }
    }

    /// Returns a reference to the texture [`Data`].
    ///
    /// [`Data`]: enum.Data.html
    pub fn data(&self) -> &Data {
        &self.data
    }

    /// Returns the origin of rotation of the texture.
    pub fn rotation_origin(&self) -> Point {
        self.rotation_origin
    }

    pub(crate) fn load_bgra(
        &self,
    ) -> Result<ImageBuffer<image::Bgra<u8>, Vec<u8>>, TextureError> {
        self.data.load_bgra()
    }
}

/// The data of a [`Texture`].
///
/// [`Texture`]: struct.Texture.html
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
    ) -> Result<ImageBuffer<image::Bgra<u8>, Vec<u8>>, TextureError> {
        match self {
            Data::Path(path) => {
                let img = match image::open(path) {
                    Ok(img) => img,
                    Err(e) => {
                        return Err(TextureError::ImageError(
                            e,
                            String::from(
                                path.to_str().unwrap_or("Invalid path"),
                            ),
                        ))
                    }
                };
                Ok(img.to_bgra())
            }
            Data::Bytes(bytes) => {
                let img = match image::load_from_memory(bytes.as_slice()) {
                    Ok(img) => img,
                    Err(e) => {
                        return Err(TextureError::ImageError(
                            e,
                            String::from(""),
                        ))
                    }
                };
                Ok(img.to_bgra())
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
                    return Err(TextureError::PixelBufferTooSmall(
                        *width, *height,
                    ));
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

pub trait IdGroup: Hash + Copy + Clone + PartialEq {
    fn hash_to_u64(&self) -> u64 {
        let mut hasher = Hasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
}
