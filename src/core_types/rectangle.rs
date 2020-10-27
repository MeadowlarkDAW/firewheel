use super::{Point, Size, Vector};

/// A rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rectangle {
    /// X coordinate of the top-left corner.
    pub x: f32,

    /// Y coordinate of the top-left corner.
    pub y: f32,

    /// Width of the rectangle.
    pub width: f32,

    /// Height of the rectangle.
    pub height: f32,
}

impl Rectangle {
    /// Creates a new [`Rectangle`] with its top-left corner in the given
    /// [`Point`] and with the provided [`Size`].
    ///
    /// [`Rectangle`]: struct.Rectangle.html
    /// [`Point`]: struct.Point.html
    /// [`Size`]: struct.Size.html
    pub fn new(top_left: Point, size: Size) -> Self {
        Self {
            x: top_left.x,
            y: top_left.y,
            width: size.width,
            height: size.height,
        }
    }

    /// Creates a new [`Rectangle`] with its top-left corner at the origin
    /// and with the provided [`Size`].
    ///
    /// [`Rectangle`]: struct.Rectangle.html
    /// [`Size`]: struct.Size.html
    pub fn with_size(size: Size) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: size.width,
            height: size.height,
        }
    }

    /// Returns the [`Point`] at the center of the [`Rectangle`].
    ///
    /// [`Point`]: struct.Point.html
    /// [`Rectangle`]: struct.Rectangle.html
    pub fn center(&self) -> Point {
        Point::new(self.center_x(), self.center_y())
    }

    /// Returns the X coordinate of the [`Point`] at the center of the
    /// [`Rectangle`].
    ///
    /// [`Point`]: struct.Point.html
    /// [`Rectangle`]: struct.Rectangle.html
    pub fn center_x(&self) -> f32 {
        self.x + self.width / 2.0
    }

    /// Returns the Y coordinate of the [`Point`] at the center of the
    /// [`Rectangle`].
    ///
    /// [`Point`]: struct.Point.html
    /// [`Rectangle`]: struct.Rectangle.html
    pub fn center_y(&self) -> f32 {
        self.y + self.height / 2.0
    }

    /// Returns the position of the top left corner of the [`Rectangle`].
    ///
    /// [`Rectangle`]: struct.Rectangle.html
    pub fn position(&self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Returns the [`Size`] of the [`Rectangle`].
    ///
    /// [`Size`]: struct.Size.html
    /// [`Rectangle`]: struct.Rectangle.html
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Returns true if the given [`Point`] is contained in the [`Rectangle`].
    ///
    /// [`Point`]: struct.Point.html
    /// [`Rectangle`]: struct.Rectangle.html
    pub fn contains(&self, point: Point) -> bool {
        self.x <= point.x
            && point.x <= self.x + self.width
            && self.y <= point.y
            && point.y <= self.y + self.height
    }

    /// Computes the intersection with the given [`Rectangle`].
    ///
    /// [`Rectangle`]: struct.Rectangle.html
    pub fn intersection(&self, other: &Rectangle) -> Option<Rectangle> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);

        let lower_right_x = (self.x + self.width).min(other.x + other.width);
        let lower_right_y = (self.y + self.height).min(other.y + other.height);

        let width = lower_right_x - x;
        let height = lower_right_y - y;

        if width > 0.0 && height > 0.0 {
            Some(Rectangle {
                x,
                y,
                width,
                height,
            })
        } else {
            None
        }
    }

    /// Snaps the [`Rectangle`] to __unsigned__ integer coordinates.
    ///
    /// [`Rectangle`]: struct.Rectangle.html
    pub fn snap(self) -> Rectangle {
        Rectangle {
            x: self.x.round(),
            y: self.y.round(),
            width: self.width.ceil(),
            height: self.height.ceil(),
        }
    }
}

impl std::ops::Mul<f32> for Rectangle {
    type Output = Self;

    fn mul(self, scale: f32) -> Self {
        Self {
            x: self.x as f32 * scale,
            y: self.y as f32 * scale,
            width: self.width * scale,
            height: self.height * scale,
        }
    }
}

impl std::ops::Add<Vector> for Rectangle {
    type Output = Rectangle;

    fn add(self, translation: Vector) -> Self {
        Rectangle {
            x: self.x + translation.x,
            y: self.y + translation.y,
            ..self
        }
    }
}
