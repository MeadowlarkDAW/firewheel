use super::texture_pipeline::Pipeline;
use crate::{Background, Color, Point};

pub struct BackgroundRenderer {
    background: Background,
}

impl BackgroundRenderer {
    pub fn new() -> Self {
        Self {
            background: Background::SolidColor(Color::BLACK),
        }
    }

    pub fn set_background(&mut self, background: Background) {
        self.background = background;
    }

    pub fn render(
        &self,
        pipeline: &mut Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        frame_view: &wgpu::TextureView,
    ) {
        // Clear the screen
        let clear_color = if let Background::SolidColor(color) = self.background
        {
            color
        } else {
            Color::BLACK
        };
        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: f64::from(clear_color.r),
                        g: f64::from(clear_color.g),
                        b: f64::from(clear_color.b),
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        match &self.background {
            Background::Texture(handle) => {
                pipeline.add_instance(handle, Point::ORIGIN, 0.0);
            }
            _ => {}
        }
    }
}
