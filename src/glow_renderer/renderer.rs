use femtovg::{Align, Baseline, Color, Paint, Path};
use glow::HasContext as _;
use std::ffi::c_void;
use std::sync::Arc;

use crate::{size::PhysicalSize, Canvas, Rect, ScaleFactor, Size};

// TODO: Pack multiple layers into a single texture instead of having one
// texture per layer.

pub struct Renderer {
    vg_canvas: femtovg::Canvas<femtovg::renderer::OpenGl>,

    physical_size: PhysicalSize,
    scale_factor: ScaleFactor,
}

impl Renderer {
    pub unsafe fn new_from_function<F>(load_fn: F) -> Self
    where
        F: FnMut(&str) -> *const c_void,
    {
        let vg_renderer = femtovg::renderer::OpenGl::new_from_function(load_fn).unwrap();
        let vg_canvas = femtovg::Canvas::new(vg_renderer).unwrap();

        Self {
            vg_canvas,
            physical_size: PhysicalSize::default(),
            scale_factor: ScaleFactor(0.0),
        }
    }

    pub fn render<MSG>(
        &mut self,
        canvas: &mut Canvas<MSG>,
        physical_size: PhysicalSize,
        scale_factor: ScaleFactor,
        clear_color: Color,
    ) {
        if self.physical_size != physical_size || self.scale_factor != scale_factor {
            self.physical_size = physical_size;
            self.scale_factor = scale_factor;

            self.vg_canvas.set_size(
                physical_size.width,
                physical_size.height,
                scale_factor.0 as f32,
            );
        }

        self.vg_canvas
            .clear_rect(0, 0, physical_size.width, physical_size.height, clear_color);

        self.vg_canvas.save();
        self.vg_canvas.reset();

        self.vg_canvas.restore();
        self.vg_canvas.flush();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // TODO: drop gl resources
    }
}
