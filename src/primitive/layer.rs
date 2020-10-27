//! Organize rendering primitives into a flattened list of layers.

use super::image;
use super::triangle;

/// A group of primitives that should be clipped together.
pub struct Layer<'a> {
    /// The clipping bounds of the [`Layer`].
    ///
    /// [`Layer`]: struct.Layer.html
    pub bounds: Rectangle,

    /// The images of the [`Layer`].
    ///
    /// [`Layer`]: struct.Layer.html
    pub images: Vec<Image>,
}

impl<'a> Layer<'a> {
    /// Creates a new [`Layer`] with the given clipping bounds.
    ///
    /// [`Layer`]: struct.Layer.html
    pub fn new(bounds: Rectangle) -> Self {
        Self {
            bounds,
            images: Vec::new(),
        }
    }

    
}