use crate::{PhySize, Point, Rect, Size};
use glam::Mat4;

#[derive(Debug)]
pub struct Viewport {
    physical_size: PhySize,
    logical_size: Size,
    scale_factor: f64,
    projection: Mat4,
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

        let projection = Mat4::orthographic_rh_gl(
            0.0,
            logical_size.width(),
            logical_size.height(),
            0.0,
            -1.0,
            1.0,
        );

        Self {
            physical_size,
            logical_size,
            scale_factor,
            projection,
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

    /// Returns the projection matrix of the [`Viewport`].
    ///
    /// [`Viewport`]: struct.Viewport.html
    pub fn projection(&self) -> &Mat4 {
        &self.projection
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
