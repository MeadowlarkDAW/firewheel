/// An amount of space in 2 dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    /// The width.
    pub width: f32,
    /// The height.
    pub height: f32,
}

impl Size {
    /// Creates a new  [`Size`] with the given width and height.
    ///
    /// [`Size`]: struct.Size.html
    pub const fn new(width: f32, height: f32) -> Self {
        Size { width, height }
    }
}

impl From<[f32; 2]> for Size {
    fn from([width, height]: [f32; 2]) -> Self {
        Size::new(width, height)
    }
}

impl From<[u16; 2]> for Size {
    fn from([width, height]: [u16; 2]) -> Self {
        Size::new(f32::from(width), f32::from(height))
    }
}

impl From<[u32; 2]> for Size {
    fn from([width, height]: [u32; 2]) -> Self {
        Size::new(width as f32, height as f32)
    }
}

impl From<Size> for [f32; 2] {
    fn from(size: Size) -> [f32; 2] {
        [size.width, size.height]
    }
}

impl From<Size> for baseview::Size {
    fn from(size: Size) -> baseview::Size {
        baseview::Size::new(size.width as f64, size.height as f64)
    }
}
