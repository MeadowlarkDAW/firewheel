/// An amount of space in 2 dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size<T> {
    /// The width.
    pub width: T,
    /// The height.
    pub height: T,
}

impl<T> Size<T> {
    /// Creates a new  [`Size`] with the given width and height.
    ///
    /// [`Size`]: struct.Size.html
    pub const fn new(width: T, height: T) -> Self {
        Size { width, height }
    }
}

impl From<[f32; 2]> for Size<f32> {
    fn from([width, height]: [f32; 2]) -> Self {
        Size::new(width, height)
    }
}

impl From<[u16; 2]> for Size<u16> {
    fn from([width, height]: [u16; 2]) -> Self {
        Size::new(width, height)
    }
}

impl From<Size<f32>> for [f32; 2] {
    fn from(size: Size<f32>) -> [f32; 2] {
        [size.width, size.height]
    }
}

impl From<Size<u16>> for [f32; 2] {
    fn from(size: Size<u16>) -> [f32; 2] {
        [f32::from(size.width), f32::from(size.height)]
    }
}

impl From<Size<u16>> for baseview::Size {
    fn from(size: Size<u16>) -> baseview::Size {
        baseview::Size::new(f64::from(size.width), f64::from(size.height))
    }
}
