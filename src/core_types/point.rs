use super::Vector;

/// A 2D point.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point<T> {
    /// The X coordinate.
    pub x: T,

    /// The Y coordinate.
    pub y: T,
}

impl<T> Point<T> {
    /// Creates a new [`Point`] with the given coordinates.
    ///
    /// [`Point`]: struct.Point.html
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Point<i16> {
    /// Computes the distance to another [`Point`].
    ///
    /// [`Point`]: struct.Point.html
    pub fn distance_i16(&self, to: Point<i16>) -> f32 {
        let a = f32::from(self.x - to.x);
        let b = f32::from(self.y - to.y);

        a.hypot(b)
    }
}

impl Point<f32> {
    /// The origin (i.e. a [`Point`] at (0.0, 0.0)).
    ///
    /// [`Point`]: struct.Point.html
    pub const ORIGIN: Point<f32> = Point::new(0.0, 0.0);

    /// Computes the distance to another [`Point`].
    ///
    /// [`Point`]: struct.Point.html
    pub fn distance_f32(&self, to: Point<f32>) -> f32 {
        let a = self.x - to.x;
        let b = self.y - to.y;

        a.hypot(b)
    }
}

impl From<Point<i16>> for [f32; 2] {
    fn from(p: Point<i16>) -> [f32; 2] {
        [f32::from(p.x), f32::from(p.y)]
    }
}

impl From<Point<u16>> for [f32; 2] {
    fn from(p: Point<u16>) -> [f32; 2] {
        [f32::from(p.x), f32::from(p.y)]
    }
}

impl From<Point<f32>> for [f32; 2] {
    fn from(p: Point<f32>) -> [f32; 2] {
        [p.x, p.y]
    }
}

impl From<Point<i16>> for Point<f32> {
    fn from(p: Point<i16>) -> Point<f32> {
        Point::<f32>::new(f32::from(p.x), f32::from(p.y))
    }
}

impl From<Point<u16>> for Point<f32> {
    fn from(p: Point<u16>) -> Point<f32> {
        Point::<f32>::new(f32::from(p.x), f32::from(p.y))
    }
}

impl From<Point<f32>> for Point<i16> {
    fn from(p: Point<f32>) -> Point<i16> {
        Point::<i16>::new(p.x.round() as i16, p.y.round() as i16)
    }
}

impl std::ops::Add<Point<f32>> for Point<f32> {
    type Output = Self;

    fn add(self, rhs: Point<f32>) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub<Point<f32>> for Point<f32> {
    type Output = Self;

    fn sub(self, rhs: Point<f32>) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
