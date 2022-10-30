use femtovg::{Color, RenderTarget};

use crate::{
    layer::BackgroundLayer,
    size::{PhysicalPoint, PhysicalRect, Point},
    PaintRegionInfo, Rect, ScaleFactor,
};

use super::TextureState;

pub(crate) struct BackgroundLayerRenderer {
    texture_state: Option<TextureState>,
}

impl BackgroundLayerRenderer {
    pub fn new() -> Self {
        Self {
            texture_state: None,
        }
    }

    pub fn render(
        &mut self,
        layer: &mut BackgroundLayer,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
        //glow_context: &mut glow::Context,
        scale_factor: ScaleFactor,
    ) {
        if layer.physical_size.width == 0 || layer.physical_size.height == 0 {
            return;
        }

        if self.texture_state.is_none() {
            self.texture_state = Some(TextureState::new(layer.physical_size, vg));
        }
        let texture_state = self.texture_state.as_mut().unwrap();

        if texture_state.physical_size != layer.physical_size {
            texture_state.resize(layer.physical_size, vg);
        }

        if layer.is_dirty {
            layer.is_dirty = false;

            vg.set_render_target(RenderTarget::Image(texture_state.texture_id));

            vg.clear_rect(
                0,
                0,
                layer.physical_size.width,
                layer.physical_size.height,
                Color::rgbaf(0.0, 0.0, 0.0, 0.0),
            );

            let assigned_region_info = PaintRegionInfo {
                rect: Rect::new(Point::new(0.0, 0.0), layer.size),
                layer_rect: Rect::new(Point::new(0.0, 0.0), layer.size),
                physical_rect: PhysicalRect {
                    pos: PhysicalPoint::new(0, 0),
                    size: layer.physical_size,
                },
                layer_physical_rect: PhysicalRect {
                    pos: PhysicalPoint::new(0, 0),
                    size: layer.physical_size,
                },
                scale_factor,
            };

            vg.save();

            layer
                .assigned_node
                .borrow_mut()
                .paint(vg, &assigned_region_info);

            vg.restore();

            vg.set_render_target(femtovg::RenderTarget::Screen);
        }

        // -- Blit the layer to the screen ---------------------------------------------------------

        /*
        unsafe {
            glow_context.enable(glow::BLEND);
            glow_context.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            glow_context.bind_texture(glow::TEXTURE_2D, texture)
        }
        */

        /*
        unsafe {
            glow_context.bind_framebuffer(
                glow::READ_FRAMEBUFFER,
                Some(texture_state.native_framebuffer),
            );
            glow_context.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);

            glow_context.blit_framebuffer(
                0,
                0,
                layer.physical_size.width as i32,
                layer.physical_size.height as i32,
                layer.physical_outer_position.x,
                layer.physical_outer_position.y,
                layer.physical_outer_position.x + layer.physical_size.width as i32,
                layer.physical_outer_position.y + layer.physical_size.height as i32,
                glow::COLOR_BUFFER_BIT,
                glow::NEAREST,
            );
        }
        */

        let mut path = femtovg::Path::new();
        path.rect(
            layer.physical_outer_position.x as f32,
            layer.physical_outer_position.y as f32,
            layer.physical_size.width as f32,
            layer.physical_size.height as f32,
        );

        let paint = femtovg::Paint::image(
            texture_state.texture_id,
            0.0,
            layer.physical_size.height as f32,
            layer.physical_size.width as f32,
            -(layer.physical_size.height as f32),
            0.0,
            1.0,
        );

        vg.fill_path(&mut path, &paint);
    }

    pub fn clean_up(
        &mut self,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
        //glow_context: &mut glow::Context,
    ) {
        if let Some(mut texture_state) = self.texture_state.take() {
            texture_state.free(vg);
        }
    }
}
