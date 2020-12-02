use crate::{PhyPoint, PhySize, Point, Rect, Size};

#[derive(Debug)]
pub struct Viewport {
    physical_size: PhySize,
    logical_size: Size,
    scale_factor: f64,
    projection_scale: [f32; 2],
}

impl Viewport {
    /// Creates a new [`Viewport`] with the given physical dimensions and scale
    /// factor.
    ///
    /// [`Viewport`]: struct.Viewport.html
    pub fn from_physical_size(
        physical_size: PhySize,
        scale_factor: f64,
    ) -> Self {
        let logical_size = Size::new(
            physical_size.width as f32 / scale_factor as f32,
            physical_size.height as f32 / scale_factor as f32,
        );

        let projection_scale = [
            2.0 / f32::from(logical_size.width()),
            -2.0 / f32::from(logical_size.height()),
        ];

        Self {
            physical_size,
            logical_size,
            scale_factor,
            projection_scale,
        }
    }

    /// Returns the physical size of the [`Viewport`].
    ///
    /// [`Viewport`]: struct.Viewport.html
    pub fn physical_size(&self) -> PhySize {
        self.physical_size
    }

    /// Returns the logical size of the [`Viewport`].
    ///
    /// [`Viewport`]: struct.Viewport.html
    pub fn logical_size(&self) -> Size {
        self.logical_size
    }

    /// Returns the scale factor of the [`Viewport`].
    ///
    /// [`Viewport`]: struct.Viewport.html
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    /// Returns the projection transformation scale of the [`Viewport`].
    ///
    /// [`Viewport`]: struct.Viewport.html
    pub fn projection_scale(&self) -> [f32; 2] {
        self.projection_scale
    }

    ///
    pub fn is_hi_dpi(&self) -> bool {
        self.scale_factor > 1.0
    }

    ///
    pub fn bounds(&self) -> Rect {
        Rect::new(Point::ORIGIN, self.logical_size)
    }
}
