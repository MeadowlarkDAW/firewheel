use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleFactor(pub f32);

impl From<f32> for ScaleFactor {
    fn from(s: f32) -> Self {
        Self(s)
    }
}

impl From<f64> for ScaleFactor {
    fn from(s: f64) -> Self {
        Self(s as f32)
    }
}

impl ScaleFactor {
    pub fn as_f32(&self) -> f32 {
        self.0
    }

    pub fn as_f64(&self) -> f64 {
        f64::from(self.0)
    }
}

/// A size in logical coordinates (points)
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Size {
    width: f32,
    height: f32,
}

impl Size {
    /// Create a new size in logical coordinates (points).
    ///
    /// If any of the given values are less than zero, then they will
    /// be set to zero.
    #[inline]
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width: width.max(0.0),
            height: height.max(0.0),
        }
    }

    /// Convert to physical size (pixels)
    #[inline]
    pub fn to_physical(&self, scale_factor: ScaleFactor) -> PhysicalSize {
        PhysicalSize {
            width: (self.width * scale_factor.as_f32()).round() as u32,
            height: (self.height * scale_factor.as_f32()).round() as u32,
        }
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    /// Set the width.
    ///
    /// If the given value is less than zero, then the width will
    /// be set to zero.
    #[inline]
    pub fn set_width(&mut self, width: f32) {
        self.width = width.max(0.0);
    }

    /// Set the height.
    ///
    /// If the given value is less than zero, then the height will
    /// be set to zero.
    #[inline]
    pub fn set_height(&mut self, height: f32) {
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

    #[inline]
    pub fn partial_eq_with_epsilon(&self, other: Size) -> bool {
        ((self.width - other.width).abs() <= f32::EPSILON)
            && ((self.height - other.height).abs() <= f32::EPSILON)
    }
}

/// A size in physical coordinates (pixels)
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalSize {
    pub width: u32,
    pub height: u32,
}

impl PhysicalSize {
    /// Create a new size in physical coordinates (pixels)
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Convert to logical size (points)
    #[inline]
    pub fn to_logical(&self, scale_factor: ScaleFactor) -> Size {
        Size {
            width: self.width as f32 / scale_factor.as_f32(),
            height: self.height as f32 / scale_factor.as_f32(),
        }
    }

    /// Convert to logical size (points) using the reciprocal of the scale factor
    #[inline]
    pub fn to_logical_from_scale_recip(&self, scale_recip: f32) -> Size {
        Size {
            width: self.width as f32 * scale_recip,
            height: self.height as f32 * scale_recip,
        }
    }
}

/// A point in logical coordinates (points)
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Create a new point in logical coordinates (points)
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Convert to physical coordinates (pixels)
    #[inline]
    pub fn to_physical(&self, scale_factor: ScaleFactor) -> PhysicalPoint {
        PhysicalPoint {
            x: (self.x * scale_factor.as_f64()).round() as i32,
            y: (self.y * scale_factor.as_f64()).round() as i32,
        }
    }

    #[inline]
    pub fn partial_eq_with_epsilon(&self, other: Point) -> bool {
        ((self.x - other.x).abs() <= f64::EPSILON) && ((self.y - other.y).abs() <= f64::EPSILON)
    }
}

impl Add<Point> for Point {
    type Output = Point;
    fn add(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Point> for Point {
    type Output = Point;
    fn sub(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl SubAssign for Point {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

/// A point in physical coordinates (pixels)
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalPoint {
    pub x: i32,
    pub y: i32,
}

impl PhysicalPoint {
    /// Create a new point in physical coordinates (pixels)
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Convert to logical coordinates (points)
    #[inline]
    pub fn to_logical(&self, scale_factor: ScaleFactor) -> Point {
        Point {
            x: f64::from(self.x) / scale_factor.as_f64(),
            y: f64::from(self.y) / scale_factor.as_f64(),
        }
    }

    /// Convert to logical coordinates (points) using the reciprocal of the scale factor
    #[inline]
    pub fn to_logical_from_scale_recip(&self, scale_recip: f64) -> Point {
        Point {
            x: f64::from(self.x) * scale_recip,
            y: f64::from(self.y) * scale_recip,
        }
    }
}

/// A rectangle in logical coordinates (points)
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pos_tl: Point,
    pos_br: Point,
    size: Size,
}

impl Rect {
    #[inline]
    pub fn new(pos: Point, size: Size) -> Self {
        Self {
            pos_tl: pos,
            pos_br: Point {
                x: pos.x + f64::from(size.width),
                y: pos.y + f64::from(size.height),
            },
            size,
        }
    }

    pub fn x(&self) -> f64 {
        self.pos_tl.x
    }

    pub fn y(&self) -> f64 {
        self.pos_tl.y
    }

    pub fn width(&self) -> f32 {
        self.size.width
    }

    pub fn height(&self) -> f32 {
        self.size.height
    }

    pub fn x2(&self) -> f64 {
        self.pos_br.x
    }

    pub fn y2(&self) -> f64 {
        self.pos_br.y
    }

    pub fn pos(&self) -> Point {
        self.pos_tl
    }

    pub fn pos_br(&self) -> Point {
        self.pos_br
    }

    pub fn size(&self) -> Size {
        self.size
    }

    #[inline]
    pub fn center_x(&self) -> f64 {
        self.pos_tl.x + (f64::from(self.size.width) / 2.0)
    }

    #[inline]
    pub fn center_y(&self) -> f64 {
        self.pos_tl.y + (f64::from(self.size.height) / 2.0)
    }

    #[inline]
    pub fn center_pos(&self) -> Point {
        Point {
            x: self.center_x(),
            y: self.center_y(),
        }
    }

    #[inline]
    pub fn set_pos(&mut self, pos: Point) {
        self.pos_tl = pos;
        self.pos_br.x = pos.x + f64::from(self.size.width);
        self.pos_br.y = pos.y + f64::from(self.size.height);
    }

    #[inline]
    pub fn set_size(&mut self, size: Size) {
        self.size = size;
        self.pos_br.x = self.pos_tl.x + f64::from(size.width);
        self.pos_br.y = self.pos_tl.y + f64::from(size.height);
    }

    #[inline]
    pub fn contains_point(&self, point: Point) -> bool {
        point.x >= self.pos_tl.x
            && point.y >= self.pos_tl.y
            && point.x <= self.pos_br.x
            && point.y <= self.pos_br.y
    }

    #[inline]
    pub fn overlaps_with_rect(&self, other: Rect) -> bool {
        self.pos_br.x >= other.pos_tl.x
            && other.pos_br.x >= self.pos_tl.x
            && self.pos_br.y >= other.pos_tl.y
            && other.pos_br.y >= self.pos_tl.y
    }

    #[inline]
    pub fn partial_eq_with_epsilon(&self, other: Rect) -> bool {
        self.pos_tl.partial_eq_with_epsilon(other.pos_tl)
            && self.pos_br.partial_eq_with_epsilon(other.pos_br)
    }

    /// Convert to physical coordinates (pixels)
    #[inline]
    pub fn to_physical(&self, scale_factor: ScaleFactor) -> PhysicalRect {
        PhysicalRect {
            pos: self.pos_tl.to_physical(scale_factor),
            size: self.size.to_physical(scale_factor),
        }
    }
}

/// A rectangle in physical coordinates (pixels)
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct PhysicalRect {
    pub pos: PhysicalPoint,
    pub size: PhysicalSize,
}

impl PhysicalRect {
    pub fn new(pos: PhysicalPoint, size: PhysicalSize) -> Self {
        Self { pos: pos, size }
    }

    #[inline]
    pub fn x2(&self) -> i32 {
        self.pos.x + self.size.width as i32
    }

    #[inline]
    pub fn y2(&self) -> i32 {
        self.pos.y + self.size.height as i32
    }

    #[inline]
    pub fn pos_br(&self) -> PhysicalPoint {
        PhysicalPoint {
            x: self.x2(),
            y: self.y2(),
        }
    }

    /// Convert to logical coordinates (points)
    #[inline]
    pub fn to_logical(&self, scale_factor: ScaleFactor) -> Rect {
        Rect::new(
            self.pos.to_logical(scale_factor),
            self.size.to_logical(scale_factor),
        )
    }

    /// Convert to logical coordinates (points) using the reciprocal of the scale factor
    #[inline]
    pub fn to_logical_from_scale_recip(&self, scale_recip: f64) -> Rect {
        Rect::new(
            self.pos.to_logical_from_scale_recip(scale_recip),
            self.size.to_logical_from_scale_recip(scale_recip as f32),
        )
    }
}

/// The `clear_rect` method in femtovg wants coordinates in `u32`, not
/// `i32`, so we use this type to correctly clear the region the next
/// time the widget needs to repaint.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub(crate) struct TextureRect {
    pub x: u32,
    pub y: u32,
    pub size: PhysicalSize,
}

impl TextureRect {
    pub fn from_physical_rect(rect: PhysicalRect) -> Self {
        let mut size = rect.size;

        let x = if rect.pos.x < 0 {
            if rect.pos.x.abs() as u32 >= rect.size.width {
                size.width = 0;
            } else {
                size.width -= rect.pos.x.abs() as u32;
            }

            0
        } else {
            rect.pos.x.abs() as u32
        };
        let y = if rect.pos.y < 0 {
            if rect.pos.y.abs() as u32 >= size.height {
                size.height = 0;
            } else {
                size.height -= rect.pos.y.abs() as u32;
            }

            0
        } else {
            rect.pos.y.abs() as u32
        };

        TextureRect { x, y, size }
    }
}
