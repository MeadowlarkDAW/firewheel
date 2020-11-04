use crate::{Hasher, Point};
use image::{ImageBuffer, ImageError};
use std::fmt::Debug;
use std::hash::{Hash, Hasher as _};
use std::path::PathBuf;
use std::sync::Arc;

pub(crate) enum HandleError {
    ImageError(ImageError, String),
    PixelBufferTooSmall(u32, u32),
}

/// A handle to texture data.
#[derive(Debug, Clone)]
pub struct Handle {
    id: u64,
    dpi_mode: DpiMode,
}

impl Handle {
    /// Creates a new [`Handle`] using only a 1x dpi resolution source.
    /// The texture will be scaled up when using higher dpi resolutions.
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn from_1x_only<T: Into<Source>>(source_1x: T) -> Self {
        let source_1x: Source = source_1x.into();

        Self {
            id: 0,
            dpi_mode: DpiMode::Only1x(source_1x),
        }
    }

    /// Creates a new [`Handle`] with only a 2x hi-dpi resolution source.
    /// The texture will be scaled down when using lower dpi resolutions.
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn from_2x_only<T: Into<Source>>(source_2x: T) -> Self {
        let source_2x: Source = source_2x.into();

        Self {
            id: 0,
            dpi_mode: DpiMode::Only2x(source_2x),
        }
    }

    /// Creates a new [`Handle`] with both a 1x dpi and 2x hi-dpi resolution source.
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn from_1x_and_2x<T: Into<Source>>(source_2x: T) -> Self {
        let source_2x: Source = source_2x.into();

        Self {
            id: 0,
            dpi_mode: DpiMode::Only2x(source_2x),
        }
    }

    pub(crate) fn set_hashed_id(&mut self, id: u64) {
        self.id = id;
    }

    pub(crate) fn hashed_id(&self) -> u64 {
        self.id
    }

    pub(crate) fn load_bgra(
        &self,
        hi_dpi: bool,
    ) -> Result<(ImageBuffer<image::Bgra<u8>, Vec<u8>>, bool, Point), HandleError>
    {
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

/// [`Handle`] sources containing data for 1x and/or 2x hi-dpi resolution
/// sources.
///
/// [`Handle`]: struct.Handle.html
#[derive(Debug, Clone)]
pub enum DpiMode {
    /// Use only a 1x dpi resolution texture.
    /// The texture will be scaled up when using higher dpi resolutions.
    Only1x(Source),
    /// Use only a 2x hi-dpi resolution texture.
    /// The texture will be scaled down when using lower dpi resolutions.
    Only2x(Source),
    /// Use both a 1x dpi and 2x hi-dpi resolution textures.
    Both {
        source_1x: Source,
        source_2x: Source,
    },
}

/// A [`Handle`] source.
///
/// [`Handle`]: struct.Handle`.html
#[derive(Debug, Clone)]
pub struct Source {
    data: Arc<Data>,
    rotation_origin: Point,
}

impl Source {
    /// Creates a texture [`Source`] pointing to the image of the given path.
    ///
    /// Makes an educated guess about the image format by examining the data in the file.
    ///
    /// [`Source`]: struct.Source.html
    pub fn from_path<T: Into<PathBuf>>(
        path: T,
        rotation_origin: Point,
    ) -> Source {
        Self::from_data(Data::Path(path.into()), rotation_origin)
    }

    /// Creates a texture [`Source`] containing the image pixels directly. This
    /// function expects the input data to be provided as a `Vec<u8>` of BGRA
    /// pixels.
    ///
    /// This is useful if you have already decoded your image.
    ///
    /// [`Source`]: struct.Source.html
    pub fn from_pixels(
        width: u32,
        height: u32,
        pixels: Vec<u8>,
        rotation_origin: Point,
    ) -> Source {
        Self::from_data(
            Data::Pixels {
                width,
                height,
                pixels,
            },
            rotation_origin,
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
    pub fn from_memory(bytes: Vec<u8>, rotation_origin: Point) -> Source {
        Self::from_data(Data::Bytes(bytes), rotation_origin)
    }

    fn from_data(data: Data, rotation_origin: Point) -> Source {
        Source {
            data: Arc::new(data),
            rotation_origin,
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
    ) -> Result<ImageBuffer<image::Bgra<u8>, Vec<u8>>, HandleError> {
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
    ) -> Result<ImageBuffer<image::Bgra<u8>, Vec<u8>>, HandleError> {
        match self {
            Data::Path(path) => {
                let img = match image::open(path) {
                    Ok(img) => img,
                    Err(e) => {
                        return Err(HandleError::ImageError(
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
                        return Err(HandleError::ImageError(
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
                    return Err(HandleError::PixelBufferTooSmall(
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

pub trait IdGroup: Hash + Copy + Clone + Into<Handle> {
    fn hash_to_u64(&self) -> u64 {
        let mut hasher = Hasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
}
