#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PhyPoint {
    pub x: i32,
    pub y: i32,
}

impl PhyPoint {
    pub const ORIGIN: PhyPoint = PhyPoint { x: 0, y: 0 };

    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PhySize {
    pub width: i32,
    pub height: i32,
}

impl PhySize {
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ORIGIN: Point = Point { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn snapped_to_int(&self) -> Point {
        Point::new(self.x.round(), self.y.round())
    }
}

impl From<Point> for [f32; 2] {
    fn from(p: Point) -> Self {
        [p.x, p.y]
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Size {
    width: f32,
    height: f32,
}

impl Size {
    /// Create a new `Size`.
    ///
    /// If `width` or `height` is less than `0.0`, then that dimension will
    /// be set to `0.0` instead.
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width: if width < 0.0 { 0.0 } else { width },
            height: if height < 0.0 { 0.0 } else { height },
        }
    }

    /// Set the width.
    ///
    /// If `width < 0.0`, then the width will be set to `0.0` instead.
    pub fn set_width(&mut self, width: f32) {
        self.width = if width < 0.0 { 0.0 } else { width };
    }

    /// Set the height.
    ///
    /// If `height < 0.0`, then the height will be set to `0.0` instead.
    pub fn set_height(&mut self, height: f32) {
        self.height = if height < 0.0 { 0.0 } else { height };
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn snapped_to_int(&self) -> Size {
        Size::new(self.width.round(), self.height.round())
    }
}

impl From<Size> for [f32; 2] {
    fn from(s: Size) -> Self {
        [s.width, s.height]
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Padding {
    pub h: f32,
    pub v: f32,
}

impl Padding {
    pub const NONE: Padding = Padding { h: 0.0, v: 0.0 };

    pub const fn new(h: f32, v: f32) -> Self {
        Self { h, v }
    }

    pub fn snapped_to_int(&self) -> Padding {
        Padding::new(self.h.round(), self.v.round())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rect {
    pub top_left: Point,
    pub size: Size,
}

impl Rect {
    pub const fn new(top_left: Point, size: Size) -> Self {
        Self { top_left, size }
    }

    pub fn from_points(p1: Point, p2: Point) -> Self {
        let (top_left, size) = if p1.x < p2.x && p1.y < p2.y {
            // p1 == top_left, p2 == bottom_right
            (p1, Size::new(p2.x - p1.x, p2.y - p1.y))
        } else if p1.x < p2.x {
            // p1 == bottom_left, p2 == top_right
            (Point::new(p1.x, p2.y), Size::new(p2.x - p1.x, p1.y - p2.y))
        } else if p1.y < p2.y {
            // p2 == bottom_left, p1 == top_right
            (Point::new(p2.x, p1.y), Size::new(p1.x - p2.x, p2.y - p1.y))
        } else {
            // p2 == top_left, p1 == bottom_right
            (p2, Size::new(p1.x - p2.x, p1.y - p2.y))
        };

        Self { top_left, size }
    }

    pub fn bottom_right(&self) -> Point {
        Point::new(
            self.top_left.x + self.size.width,
            self.top_left.y + self.size.height,
        )
    }

    pub fn center(&self) -> Point {
        Point::new(
            self.top_left.x + (self.size.width / 2.0),
            self.top_left.y + (self.size.height / 2.0),
        )
    }

    pub fn snapped_to_int(&self) -> Rect {
        Rect::new(self.top_left.snapped_to_int(), self.size.snapped_to_int())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_from_points() {
        let rect1 = Rect::new(Point::new(0.0, 0.0), Size::new(1.0, 2.0))
            .snapped_to_int();

        assert_eq!(
            Rect::from_points(Point::new(0.0, 0.0), Point::new(1.0, 2.0))
                .snapped_to_int(),
            rect1
        );
        assert_eq!(
            Rect::from_points(Point::new(1.0, 0.0), Point::new(0.0, 2.0))
                .snapped_to_int(),
            rect1
        );
        assert_eq!(
            Rect::from_points(Point::new(0.0, 2.0), Point::new(1.0, 0.0))
                .snapped_to_int(),
            rect1
        );
        assert_eq!(
            Rect::from_points(Point::new(1.0, 2.0), Point::new(0.0, 0.0))
                .snapped_to_int(),
            rect1
        );
    }
}
