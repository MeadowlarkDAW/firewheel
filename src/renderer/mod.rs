use femtovg::Color;
use std::ffi::c_void;

use crate::{size::PhysicalSize, Canvas, ScaleFactor};

mod layer_renderer;
pub(crate) use layer_renderer::LayerRenderer;

// TODO: Pack multiple layers into a single texture instead of having one
// texture per layer.

pub struct Renderer {
    vg: femtovg::Canvas<femtovg::renderer::OpenGl>,

    physical_size: PhysicalSize,
    scale_factor: ScaleFactor,
}

impl Renderer {
    pub unsafe fn new_from_function<F>(load_fn: F) -> Self
    where
        F: FnMut(&str) -> *const c_void,
    {
        let vg_renderer = femtovg::renderer::OpenGl::new_from_function(load_fn).unwrap();
        let vg = femtovg::Canvas::new(vg_renderer).unwrap();

        Self {
            vg,
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
        for mut layer_renderer in canvas.layer_renderers_to_clean_up.drain(..) {
            layer_renderer.clean_up(&mut self.vg);
        }

        if self.physical_size != physical_size || self.scale_factor != scale_factor {
            self.physical_size = physical_size;
            self.scale_factor = scale_factor;

            self.vg.set_size(
                physical_size.width,
                physical_size.height,
                // Widgets will do their own dpi scaling in order to have more control
                // over snapping to whole pixels.
                1.0,
            );
        }

        self.vg
            .clear_rect(0, 0, physical_size.width, physical_size.height, clear_color);

        self.vg.save();
        self.vg.reset();

        self.vg.restore();
        self.vg.flush();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // TODO: drop gl resources
    }
}
