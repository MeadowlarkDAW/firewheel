#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleFactor(pub f64);

/// A size in logical coordinates
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Size {
    width: f64,
    height: f64,
}

impl Size {
    /// Create a new size in logical coordinates.
    ///
    /// If any of the given values are less than zero, then they will
    /// be set to zero.
    #[inline]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width: width.max(0.0),
            height: height.max(0.0),
        }
    }

    /// Convert to actual physical size
    #[inline]
    pub fn to_physical(&self, scale: ScaleFactor) -> PhysicalSize {
        PhysicalSize {
            width: (self.width * scale.0).round() as u32,
            height: (self.height * scale.0).round() as u32,
        }
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    /// Set the width.
    ///
    /// If the given value is less than zero, then the width will
    /// be set to zero.
    #[inline]
    pub fn set_width(&mut self, width: f64) {
        self.width = width.max(0.0);
    }

    /// Set the height.
    ///
    /// If the given value is less than zero, then the height will
    /// be set to zero.
    #[inline]
    pub fn set_height(&mut self, height: f64) {
        self.height = height.max(0.0);
    }

    #[inline]
    pub fn min(&self, other: Self) -> Self {
        Self {
            width: self.width.min(other.width),
            height: self.height.min(other.height),
        }
    }

    #[inline]
    pub fn max(&self, other: Self) -> Self {
        Self {
            width: self.width.max(other.width),
            height: self.height.max(other.height),
        }
    }
}

/// An actual size in physical coordinates
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalSize {
    pub width: u32,
    pub height: u32,
}

impl PhysicalSize {
    /// Create a new size in actual physical coordinates
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Convert to logical size
    #[inline]
    pub fn to_logical(&self, scale: ScaleFactor) -> Size {
        Size {
            width: f64::from(self.width) / scale.0,
            height: f64::from(self.height) / scale.0,
        }
    }

    /// Convert to logical size using the reciprocal of the scale factor
    #[inline]
    pub fn to_logical_from_scale_recip(&self, scale_recip: f64) -> Size {
        Size {
            width: f64::from(self.width) * scale_recip,
            height: f64::from(self.height) * scale_recip,
        }
    }
}

/// A point in logical coordinates
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Create a new point in logical coordinates
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Convert to actual physical coordinates
    #[inline]
    pub fn to_physical(&self, scale: ScaleFactor) -> PhysicalPoint {
        PhysicalPoint {
            x: (self.x * scale.0).round() as i32,
            y: (self.y * scale.0).round() as i32,
        }
    }
}

/// A point in actual physical coordinates
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalPoint {
    pub x: i32,
    pub y: i32,
}

impl PhysicalPoint {
    /// Create a new point in actual physical coordinates
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Convert to logical coordinates
    #[inline]
    pub fn to_logical(&self, scale: ScaleFactor) -> Point {
        Point {
            x: f64::from(self.x) / scale.0,
            y: f64::from(self.y) / scale.0,
        }
    }

    /// Convert to logical size using the reciprocal of the scale factor
    #[inline]
    pub fn to_logical_from_scale_recip(&self, scale_recip: f64) -> Point {
        Point {
            x: f64::from(self.x) * scale_recip,
            y: f64::from(self.y) * scale_recip,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub pos: Point,
    pub size: Size,
}

impl Rect {
    pub fn x(&self) -> f64 {
        self.pos.x
    }

    pub fn y(&self) -> f64 {
        self.pos.y
    }

    pub fn width(&self) -> f64 {
        self.size.width
    }

    pub fn height(&self) -> f64 {
        self.size.height
    }

    #[inline]
    pub fn x2(&self) -> f64 {
        self.pos.x + self.size.width
    }

    #[inline]
    pub fn y2(&self) -> f64 {
        self.pos.y + self.size.height
    }

    #[inline]
    pub fn pos2(&self) -> Point {
        Point {
            x: self.x2(),
            y: self.y2(),
        }
    }

    #[inline]
    pub fn center_x(&self) -> f64 {
        self.pos.x + (self.size.width / 2.0)
    }

    #[inline]
    pub fn center_y(&self) -> f64 {
        self.pos.y + (self.size.height / 2.0)
    }

    #[inline]
    pub fn center_pos(&self) -> Point {
        Point {
            x: self.center_x(),
            y: self.center_y(),
        }
    }
}
