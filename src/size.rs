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

/// A rectangle in logical coordinates
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct RegionRect {
    pub pos: Point,
    pub size: Size,
}

impl RegionRect {
    /// Construct a new rectangle describing a region
    ///
    /// If the given `width` or `height` is less than zero, than that
    /// width/height will be set to zero.
    #[inline]
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        RegionRect {
            pos: Point { x, y },
            size: Size::new(width, height),
        }
    }

    /// Construct a new rectangle describing a region from two points.
    ///
    /// The resulting position of the rectangle will be equal to `pos1`.
    ///
    /// If `pos2` has an x coordinate less than `pos1`'s x coordinate,
    /// then the width will be set to zero. Likewise if `pos2` has a
    /// y coordinate less than `pos1`'s y coordinate, then the height
    /// will be set to zero.
    #[inline]
    pub fn new_from_two_points(pos1: Point, pos2: Point) -> Self {
        Self::new(pos1.x, pos1.y, pos2.x - pos1.x, pos2.y - pos1.y)
    }

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
    pub fn pos2(&self) -> Point {
        Point {
            x: self.pos.x + self.size.width,
            y: self.pos.y + self.size.height,
        }
    }

    /// Retrieve the x coordinate of the second point.
    #[inline]
    pub fn x2(&self) -> f64 {
        self.pos.x + self.size.width
    }

    /// Retrieve the y coordinate of the second point.
    #[inline]
    pub fn y2(&self) -> f64 {
        self.pos.y + self.size.height
    }

    pub fn set_x(&mut self, x: f64) {
        self.pos.x = x;
    }

    pub fn set_y(&mut self, y: f64) {
        self.pos.y = y;
    }

    /// Set the width of the rectangle.
    ///
    /// If the given value is less than zero, then the width will
    /// be set to zero.
    #[inline]
    pub fn set_width(&mut self, width: f64) {
        self.size.set_width(width);
    }

    /// Set the height of the rectangle.
    ///
    /// If the given value is less than zero, then the height will
    /// be set to zero.
    #[inline]
    pub fn set_height(&mut self, height: f64) {
        self.size.set_height(height);
    }

    /// Set the size of the rectangle based on the second point.
    ///
    /// If `pos2` has an x coordinate less than this rectangle's x
    /// coordinate, then the width will be set to zero. Likewise if
    /// `pos2` has a y coordinate less than this rectangle's y
    /// coordinate, then the height will be set to zero.
    pub fn set_pos2(&mut self, pos2: Point) {
        self.size = Size::new(pos2.x - self.pos.x, pos2.y - self.pos.y);
    }
}
