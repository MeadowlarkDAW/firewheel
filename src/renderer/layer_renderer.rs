use femtovg::{Color, ImageFlags, ImageId, Paint, Path, PixelFormat, RenderTarget};

use crate::{
    layer::Layer,
    size::{PhysicalPoint, PhysicalRect, PhysicalSize, TextureRect},
    PaintRegionInfo, Rect, ScaleFactor,
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
        let physical_size = layer.region_tree.layer_physical_size();
        let layer_physical_internal_offset = layer.region_tree.layer_physical_internal_offset();
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
                layer.region_tree.texture_rects_to_clear.clear();

                vg.clear_rect(
                    0,
                    0,
                    physical_size.width,
                    physical_size.height,
                    Color::rgba(0, 0, 0, 0),
                );
            } else {
                for clear_rect in layer.region_tree.texture_rects_to_clear.drain(..) {
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

            let mut assigned_region_info = PaintRegionInfo {
                rect: Rect::default(),
                layer_rect: layer.region_tree.layer_rect(),
                physical_rect: PhysicalRect::default(),
                layer_physical_rect: PhysicalRect {
                    // Remove the layer's internal offset from the physical region so
                    // it is in the correct place in the texture.
                    pos: PhysicalPoint::new(0, 0),
                    size: physical_size,
                },
                scale_factor,
            };
            for widget_entry in layer.region_tree.dirty_widgets.iter_mut() {
                vg.save();

                if let Some(assigned_region) = widget_entry.assigned_region().upgrade() {
                    let (assigned_rect, physical_rect) = {
                        let mut assigned_region = assigned_region.borrow_mut();

                        // Remove the layer's internal offset from the physical region so
                        // it is in the correct place in the texture.
                        let mut physical_rect = assigned_region.region.physical_rect;
                        physical_rect.pos.x -= layer_physical_internal_offset.x;
                        physical_rect.pos.y -= layer_physical_internal_offset.y;

                        // The `clear_rect` method in femtovg wants coordinates in `u32`, not
                        // `i32`, so we use this type to correctly clear the region the next
                        // time the widget needs to repaint.
                        let texture_rect = TextureRect::from_physical_rect(physical_rect);
                        assigned_region.region.last_rendered_texture_rect = Some(texture_rect);

                        (assigned_region.region.rect, physical_rect)
                    };

                    assigned_region_info.rect = assigned_rect;
                    assigned_region_info.physical_rect = physical_rect;

                    widget_entry.borrow_mut().paint(vg, &assigned_region_info);
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
            layer.physical_outer_position.x as f32,
            layer.physical_outer_position.y as f32,
            physical_size.width as f32,
            physical_size.height as f32,
        );

        let paint = Paint::image(
            texture_state.texture_id,
            0.0,
            0.0,
            physical_size.width as f32,
            physical_size.height as f32,
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
