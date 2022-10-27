use femtovg::{ImageFlags, ImageId, PixelFormat};
use std::ffi::c_void;

use crate::{layer::StrongLayerEntry, size::PhysicalSize, AppWindow, ScaleFactor};

mod background_layer_renderer;
mod widget_layer_renderer;
pub(crate) use background_layer_renderer::BackgroundLayerRenderer;
pub(crate) use widget_layer_renderer::WidgetLayerRenderer;

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
        app_window: &mut AppWindow<MSG>,
        physical_size: PhysicalSize,
        scale_factor: ScaleFactor,
    ) {
        for mut layer_renderer in app_window.widget_layer_renderers_to_clean_up.drain(..) {
            layer_renderer.clean_up(&mut self.vg);
        }
        for mut layer_renderer in app_window.background_layer_renderers_to_clean_up.drain(..) {
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

        for (_z_order, layer_entries) in app_window.layers_ordered.iter_mut() {
            for layer_entry in layer_entries.iter_mut() {
                match layer_entry {
                    StrongLayerEntry::Widget(layer_entry) => {
                        let mut layer = layer_entry.borrow_mut();
                        if layer.is_visible() {
                            let mut layer_renderer = layer.renderer.take().unwrap();

                            layer_renderer.render(&mut *layer, &mut self.vg, scale_factor);

                            layer.renderer = Some(layer_renderer);
                        }
                    }
                    StrongLayerEntry::Background(layer_entry) => {
                        let mut layer = layer_entry.borrow_mut();
                        if layer.is_visible() {
                            let mut layer_renderer = layer.renderer.take().unwrap();

                            layer_renderer.render(&mut *layer, &mut self.vg, scale_factor);

                            layer.renderer = Some(layer_renderer);
                        }
                    }
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

struct TextureState {
    texture_id: ImageId,
    physical_size: PhysicalSize,
}

impl TextureState {
    fn new(
        physical_size: PhysicalSize,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
    ) -> Self {
        let texture_id = vg
            .create_image_empty(
                physical_size.width as usize,
                physical_size.height as usize,
                PixelFormat::Rgba8,
                ImageFlags::NEAREST,
            )
            .unwrap();

        Self {
            texture_id,
            physical_size,
        }
    }

    fn resize(
        &mut self,
        physical_size: PhysicalSize,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
    ) {
        vg.realloc_image(
            self.texture_id,
            physical_size.width as usize,
            physical_size.height as usize,
            PixelFormat::Rgba8,
            ImageFlags::NEAREST,
        )
        .unwrap();
    }
}
