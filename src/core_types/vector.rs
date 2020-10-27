/// A 2D vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector {
    /// The X component of the [`Vector`]
    ///
    /// [`Vector`]: struct.Vector.html
    pub x: f32,

    /// The Y component of the [`Vector`]
    ///
    /// [`Vector`]: struct.Vector.html
    pub y: f32,
}

impl Vector {
    /// Creates a new [`Vector`] with the given components.
    ///
    /// [`Vector`]: struct.Vector.html
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl std::ops::Add for Vector {
    type Output = Self;

    fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y)
    }
}

impl std::ops::Sub for Vector {
    type Output = Self;

    fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y)
    }
}

impl std::ops::Mul<f32> for Vector {
    type Output = Self;

    fn mul(self, scale: f32) -> Self {
        Self::new(self.x * scale, self.y * scale)
    }
}

impl Default for Vector {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}
