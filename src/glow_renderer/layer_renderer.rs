use glow::{Context, HasContext, NativeFramebuffer, NativeProgram, NativeTexture, Program};
use std::sync::Arc;

use crate::{layer::Layer, size::PhysicalSize, ScaleFactor, Size};

// TODO: Pack multiple layers into a single texture instead of having one
// texture per layer.

pub(crate) struct LayerRenderer {}

impl LayerRenderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render<MSG>(&mut self, layer: &mut Layer<MSG>, scale_factor: ScaleFactor) {
        let physical_size = layer.region_tree.layer_size().to_physical(scale_factor);
        if physical_size.width == 0 || physical_size.height == 0 {
            return;
        }
    }
}
