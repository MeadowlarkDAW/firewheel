use crate::wgpu_renderer::Renderer;
use crate::{atlas, Size, TextureHandle, Window};
use futures::executor::block_on;

pub struct Canvas {
    pub(crate) renderer: Renderer,
    texture_handles: Vec<TextureHandle>,
}

impl Canvas {
    pub(crate) fn new(window: &mut Window) -> Self {
        let physical_size = Size::new(
            window.window_info().physical_size().width as f32,
            window.window_info().physical_size().height as f32,
        );

        let renderer = block_on(Renderer::new(
            window,
            physical_size,
            window.window_info().scale(),
        ))
        .unwrap();

        Self {
            renderer,
            texture_handles: Vec::new(),
        }
    }

    /// Replace the current texture atlas with a new one.
    ///
    /// If this operation fails, the current texture atlas may be corrupt.
    /// Please load your default texture atlas again if an error ocurred.
    pub fn replace_texture_atlas<T: Into<TextureHandle> + Copy + Clone>(
        &mut self,
        textures: &[T],
    ) -> Result<(), atlas::AtlasError> {
        self.texture_handles = textures
            .iter()
            .map(|texture| -> TextureHandle { (*texture).into() })
            .collect();

        self.renderer
            .replace_texture_atlas(self.texture_handles.as_slice())
    }

    // TODO:
    //   - request_scale_factor(&mut self, scale_factor: f64);
}
