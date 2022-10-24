use femtovg::{Color, ImageFlags, ImageId, Paint, Path, PixelFormat, RenderTarget};

use crate::{
    layer::Layer,
    size::{PhysicalRect, PhysicalSize},
    ScaleFactor,
};

// TODO: Pack multiple layers into a single texture instead of having one
// texture per layer.

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

pub(crate) struct LayerRenderer {
    texture_state: Option<TextureState>,
}

impl LayerRenderer {
    pub fn new() -> Self {
        Self {
            texture_state: None,
        }
    }

    pub fn render<MSG>(
        &mut self,
        layer: &mut Layer<MSG>,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
        scale_factor: ScaleFactor,
    ) {
        let physical_size = layer.region_tree.layer_size().to_physical(scale_factor);
        if physical_size.width == 0 || physical_size.height == 0 {
            return;
        }

        if self.texture_state.is_none() {
            self.texture_state = Some(TextureState::new(physical_size, vg));
        }
        let texture_state = self.texture_state.as_mut().unwrap();

        if texture_state.physical_size != physical_size {
            texture_state.resize(physical_size, vg);
        }

        if layer.is_dirty() {
            vg.set_render_target(RenderTarget::Image(texture_state.texture_id));

            // -- Clear the regions marked to be cleared -------------------------------------------

            if layer.region_tree.clear_whole_layer {
                layer.region_tree.clear_whole_layer = false;
                layer.region_tree.physical_rects_to_clear.clear();

                vg.clear_rect(
                    0,
                    0,
                    physical_size.width,
                    physical_size.height,
                    Color::rgba(0, 0, 0, 0),
                );
            } else {
                for clear_rect in layer.region_tree.physical_rects_to_clear.drain(..) {
                    if clear_rect.size.width == 0 || clear_rect.size.height == 0 {
                        continue;
                    }

                    vg.clear_rect(
                        clear_rect.x,
                        clear_rect.y,
                        clear_rect.size.width,
                        clear_rect.size.height,
                        Color::rgba(0, 0, 0, 0),
                    );
                }
            }

            // -- Paint the dirty widgets ----------------------------------------------------------

            let layer_rect = layer.region_tree.layer_rect();
            for widget_entry in layer.region_tree.dirty_widgets.iter_mut() {
                vg.save();

                if let Some(assigned_region) = widget_entry.assigned_region().upgrade() {
                    let assigned_rect = {
                        let mut assigned_region = assigned_region.borrow_mut();

                        // Mark the physical region where the widget is painting to so it can
                        // be cleared the next time the widget needs to repaint.
                        let rendered_pos = assigned_region.region.rect.pos() - layer_rect.pos();
                        assigned_region.region.last_rendered_physical_rect =
                            Some(PhysicalRect::from_logical_pos_size(
                                rendered_pos,
                                assigned_region.region.rect.size(),
                                scale_factor,
                            ));

                        assigned_region.region.rect
                    };

                    widget_entry
                        .borrow_mut()
                        .paint(vg, &assigned_rect, &layer_rect);
                } else {
                    log::error!("Someting went wrong: widget was not assigned a region");
                }

                vg.restore();
            }
            layer.region_tree.dirty_widgets.clear();

            vg.set_render_target(RenderTarget::Screen);
        }

        // -- Blit the layer to the screen ---------------------------------------------------------

        let mut path = Path::new();
        path.rect(
            layer.outer_position.x as f32,
            layer.outer_position.y as f32,
            layer.region_tree.layer_size().width() as f32,
            layer.region_tree.layer_size().height() as f32,
        );

        let paint = Paint::image(
            texture_state.texture_id,
            0.0,
            0.0,
            layer.region_tree.layer_size().width() as f32,
            layer.region_tree.layer_size().height() as f32,
            0.0,
            1.0,
        );

        vg.fill_path(&mut path, &paint);
    }

    pub fn clean_up(&mut self, vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>) {
        if let Some(texture_state) = self.texture_state.take() {
            vg.delete_image(texture_state.texture_id);
        }
    }
}
