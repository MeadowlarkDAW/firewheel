use crate::renderer::Renderer;
use crate::{atlas, texture, Background};

pub struct Root<'a, 'b> {
    window: &'a mut baseview::Window<'b>,
    renderer: &'a mut Renderer,
}

impl<'a, 'b> Root<'a, 'b> {
    pub(crate) fn new(
        window: &'a mut baseview::Window<'b>,
        renderer: &'a mut Renderer,
    ) -> Self {
        Self { window, renderer }
    }

    /// Replace the current texture atlas with a new one.
    ///
    /// If this operation fails, the current texture atlas may be corrupt.
    /// Please load your default texture atlas again if an error ocurred.
    pub fn new_texture_atlas(
        &mut self,
        texture_loaders: &mut [texture::Loader],
    ) -> Result<(), atlas::AtlasError> {
        self.renderer.new_texture_atlas(texture_loaders)
    }

    /// Set the window background from one or multiple multiple backgrounds.
    pub fn set_background(&mut self, background: Background) {
        self.renderer.set_background(background);
    }

    // TODO:
    //   - request_scale_factor(&mut self, scale_factor: f64);
}
