use glow::HasContext as _;
use std::sync::Arc;

use crate::{size::PhysicalSize, Canvas, ScaleFactor, Size};

use super::layer_renderer::LayerRenderer;

pub struct CanvasRenderer {
    gl: Arc<glow::Context>,
    canvas_size: Size,
    physical_size: PhysicalSize,
    scale_factor: ScaleFactor,
    clear_color: [f32; 4],

    layers: Vec<LayerRenderer>,
}

impl CanvasRenderer {
    pub fn new(gl: Arc<glow::Context>, canvas_size: Size, scale_factor: ScaleFactor) -> Self {
        let physical_size = canvas_size.to_physical(scale_factor);

        Self {
            gl,
            canvas_size,
            physical_size,
            scale_factor,
            clear_color: [0.0, 0.0, 0.0, 1.0],
            layers: Vec::new(),
        }
    }

    pub fn render<MSG>(&mut self, canvas: &mut Canvas<MSG>) {
        // TODO: Check if canvas size has changed.

        self.clear();
    }

    fn clear(&mut self) {
        unsafe {
            self.gl.disable(glow::SCISSOR_TEST);

            self.gl.viewport(
                0,
                0,
                self.physical_size.width as i32,
                self.physical_size.height as i32,
            );

            self.gl.clear_color(
                self.clear_color[0],
                self.clear_color[1],
                self.clear_color[2],
                self.clear_color[3],
            );

            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
        }
    }
}
