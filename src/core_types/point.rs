use super::Vector;

/// A 2D point.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    /// The X coordinate.
    pub x: f32,

    /// The Y coordinate.
    pub y: f32,
}

impl Point {
    /// The origin (i.e. a [`Point`] at (0, 0)).
    ///
    /// [`Point`]: struct.Point.html
    pub const ORIGIN: Point = Point::new(0.0, 0.0);

    /// Creates a new [`Point`] with the given coordinates.
    ///
    /// [`Point`]: struct.Point.html
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Computes the distance to another [`Point`].
    ///
    /// [`Point`]: struct.Point.html
    pub fn distance(&self, to: Point) -> f32 {
        let a = self.x - to.x;
        let b = self.y - to.y;

        a.hypot(b)
    }

    /// Snaps the [`Point`] to integer coordinates.
    ///
    /// [`Point`]: struct.Point.html
    pub fn snap(&self) -> Point {
        Point::new(self.x.round(), self.y.round())
    }
}

impl From<[f32; 2]> for Point {
    fn from([x, y]: [f32; 2]) -> Self {
        Point { x, y }
    }
}

impl From<[u16; 2]> for Point {
    fn from([x, y]: [u16; 2]) -> Self {
        Point::new(x.into(), y.into())
    }
}

impl From<[u32; 2]> for Point {
    fn from([x, y]: [u32; 2]) -> Self {
        Point::new(x as f32, y as f32)
    }
}

impl From<Point> for [f32; 2] {
    fn from(point: Point) -> [f32; 2] {
        [point.x, point.y]
    }
}

impl std::ops::Add<Vector> for Point {
    type Output = Self;

    fn add(self, vector: Vector) -> Self {
        Self {
            x: self.x + vector.x,
            y: self.y + vector.y,
        }
    }
}

impl std::ops::Sub<Vector> for Point {
    type Output = Self;

    fn sub(self, vector: Vector) -> Self {
        Self {
            x: self.x - vector.x,
            y: self.y - vector.y,
        }
    }
}

impl std::ops::Sub<Point> for Point {
    type Output = Vector;

    fn sub(self, point: Point) -> Vector {
        Vector::new(self.x - point.x, self.y - point.y)
    }
}
