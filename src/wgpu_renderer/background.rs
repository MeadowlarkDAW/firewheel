use crate::wgpu_renderer::texture_pipeline::Pipeline;
use crate::{Background, Color, Point, Rectangle};

pub struct BackgroundRenderer {
    background: Background,
    redraw_areas: Vec<Rectangle>,

    do_full_redraw: bool,
}

impl BackgroundRenderer {
    pub fn new() -> Self {
        Self {
            background: Background::SolidColor(Color::BLACK),
            // Probably won't need more than this.
            redraw_areas: Vec::with_capacity(10),
            do_full_redraw: true,
        }
    }

    pub fn set_background(&mut self, background: Background) {
        self.background = background;
        self.do_full_redraw = true;
    }

    pub fn changed(&self) -> bool {
        self.do_full_redraw || !self.redraw_areas.is_empty()
    }

    pub fn queue_full_redraw(&mut self) {
        self.do_full_redraw = true;
    }

    pub fn queue_redraw_area(&mut self, area: Rectangle) {
        self.redraw_areas.push(area);
    }

    pub fn render(
        &mut self,
        pipeline: &mut Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        frame_view: &wgpu::TextureView,
    ) {
        if self.do_full_redraw {
            // Redraw areas are irrelevant when doing a full redraw.
            self.redraw_areas.clear();

            self.full_redraw(pipeline, encoder, frame_view);

            self.do_full_redraw = false;
            return;
        }

        if self.redraw_areas.is_empty() {
            return;
        }

        match &self.background {
            Background::SolidColor(color) => {
                // TODO: Draw colored rectangles into areas
            }
            Background::Texture(id) => {
                for area in &self.redraw_areas {
                    pipeline.add_clipped_instance(*id, Point::new(0, 0), *area);
                }
            }
            Background::MultipleTextures(ids) => {
                for (id, position) in ids {
                    for area in &self.redraw_areas {
                        pipeline.add_clipped_instance(*id, *position, *area);
                    }
                }
            }
        }
    }

    fn full_redraw(
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
            Background::Texture(id) => {
                pipeline.add_instance(*id, Point::new(0, 0), 0.0);
            }
            Background::MultipleTextures(ids) => {
                for (id, position) in ids {
                    pipeline.add_instance(*id, *position, 0.0);
                }
            }
            _ => {}
        }
    }
}
