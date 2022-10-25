use femtovg::Color;
use std::ffi::c_void;

use crate::{size::PhysicalSize, ScaleFactor, WindowCanvas};

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
        window_canvas: &mut WindowCanvas<MSG>,
        physical_size: PhysicalSize,
        scale_factor: ScaleFactor,
    ) {
        for mut layer_renderer in window_canvas.layer_renderers_to_clean_up.drain(..) {
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

        self.vg.save();
        self.vg.reset();

        for (_z_order, layer_entries) in window_canvas.layers_ordered.iter_mut() {
            for layer_entry in layer_entries.iter_mut() {
                let mut layer = layer_entry.borrow_mut();
                if layer.is_visible() {
                    let mut layer_renderer = layer.renderer.take().unwrap();

                    layer_renderer.render(&mut *layer, &mut self.vg, scale_factor);

                    layer.renderer = Some(layer_renderer);
                }
            }
        }

        self.vg.restore();
        self.vg.flush();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // TODO: drop gl resources
    }
}
