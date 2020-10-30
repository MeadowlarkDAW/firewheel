use crate::{Hasher, Point};
use image::{ImageBuffer, ImageError};
use std::fmt::Debug;
use std::hash::{Hash, Hasher as _};
use std::path::PathBuf;
use std::sync::Arc;

pub enum TextureSourceError {
    ImageError(ImageError),
    PixelBufferTooSmall(u32, u32),
    NoData,
}

impl From<ImageError> for TextureSourceError {
    fn from(e: ImageError) -> Self {
        TextureSourceError::ImageError(e)
    }
}

#[derive(Debug)]
pub struct TextureSource {
    id: u64,
    handle: Option<Handle>,
    handle_2x: Option<Handle>,
    r_origin: Point,
}

impl TextureSource {
    /// Creates a new [`Texture`] with the given path.
    ///
    /// [`Image`]: struct.Image.html
    pub fn new<T: Into<Handle>>(
        handle: Option<T>,
        handle_2x: Option<T>,
        rotation_origin: Point,
    ) -> Self {
        let handle = handle.map(|handle| handle.into());
        let handle_2x = handle_2x.map(|handle| handle.into());

        let mut hasher = Hasher::default();
        if let Some(handle) = &handle {
            handle.data().hash(&mut hasher);
        }
        if let Some(handle) = &handle_2x {
            handle.data().hash(&mut hasher);
        }

        Self {
            id: hasher.finish(),
            handle,
            handle_2x,
            r_origin: rotation_origin,
        }
    }

    /// Returns the rotation origin (for 1x scale) of this texture
    pub fn rotation_origin(&self) -> Point {
        self.r_origin
    }

    /// Get the handle to the texture. If the texture does not exist for the given dpi, then it will try the other one.
    ///
    /// It will also return whether the returned handle is marked as hi-dpi or not.
    fn handle(&self, hi_dpi: bool) -> (&Option<Handle>, bool) {
        if self.handle_2x.is_some() {
            if hi_dpi {
                (&self.handle_2x, true)
            } else if self.handle.is_none() {
                (&self.handle_2x, true)
            } else {
                (&self.handle, false)
            }
        } else {
            (&self.handle, false)
        }
    }

    pub(crate) fn load_bgra(
        &self,
        hi_dpi: bool,
    ) -> Result<(ImageBuffer<image::Bgra<u8>, Vec<u8>>, bool), TextureSourceError>
    {
        let (handle, hi_dpi) = self.handle(hi_dpi);

        if let Some(handle) = handle {
            Ok((handle.load_bgra()?, hi_dpi))
        } else {
            Err(TextureSourceError::NoData)
        }
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }
}

/// A [`Texture`] handle.
///
/// [`Texture`]: struct.Texture.html
#[derive(Debug, Clone)]
pub struct Handle {
    data: Arc<Data>,
}

impl Handle {
    /// Creates a texture [`Handle`] pointing to the image of the given path.
    ///
    /// Makes an educated guess about the image format by examining the data in the file.
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn from_path<T: Into<PathBuf>>(path: T) -> Handle {
        Self::from_data(Data::Path(path.into()))
    }

    /// Creates a texture [`Handle`] containing the image pixels directly. This
    /// function expects the input data to be provided as a `Vec<u8>` of BGRA
    /// pixels.
    ///
    /// This is useful if you have already decoded your image.
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn from_pixels(width: u32, height: u32, pixels: Vec<u8>) -> Handle {
        Self::from_data(Data::Pixels {
            width,
            height,
            pixels,
        })
    }

    /// Creates a texture [`Handle`] containing the image data directly.
    ///
    /// Makes an educated guess about the image format by examining the given data.
    ///
    /// This is useful if you already have your image loaded in-memory, maybe
    /// because you downloaded or generated it procedurally.
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn from_memory(bytes: Vec<u8>) -> Handle {
        Self::from_data(Data::Bytes(bytes))
    }

    fn from_data(data: Data) -> Handle {
        Handle {
            data: Arc::new(data),
        }
    }

    /// Returns a reference to the image [`Data`].
    ///
    /// [`Data`]: enum.Data.html
    pub fn data(&self) -> &Data {
        &self.data
    }

    pub(crate) fn load_bgra(
        &self,
    ) -> Result<ImageBuffer<image::Bgra<u8>, Vec<u8>>, TextureSourceError> {
        self.data.load_bgra()
    }
}

/// The data of a [`Texture`].
///
/// [`Texture`]: struct.Texture.html
#[derive(Clone, Hash)]
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
    pub fn load_bgra(
        &self,
    ) -> Result<ImageBuffer<image::Bgra<u8>, Vec<u8>>, TextureSourceError> {
        match self {
            Data::Path(path) => {
                let img = image::open(path)?;
                Ok(img.to_bgra())
            }
            Data::Bytes(bytes) => {
                let img = image::load_from_memory(bytes.as_slice())?;
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
                    return Err(TextureSourceError::PixelBufferTooSmall(
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
