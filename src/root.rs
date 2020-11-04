use crate::wgpu_renderer::Renderer;
use crate::{atlas, texture, Background, Color, Size, Window};
use futures::executor::block_on;
use std::collections::HashMap;

pub struct Root<T: texture::IdGroup> {
    pub(crate) renderer: Renderer,
    background: Background<T>,
    do_full_redraw: bool,
}

impl<T: texture::IdGroup> Root<T> {
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
            background: Background::SolidColor(Color::BLACK),
            do_full_redraw: true,
        }
    }

    pub(crate) fn window_resized(&mut self, physical_size: Size, scale: f64) {
        self.renderer.resize(physical_size, scale);
        self.do_full_redraw = true;
    }

    pub(crate) fn render(&mut self) {
        if self.do_full_redraw {
            let clear_color = match self.background {
                Background::SolidColor(color) => Some(color),
                _ => None,
            };

            self.renderer.render(self.do_full_redraw, &self.background);

            self.do_full_redraw = false;
        }
    }

    /// Replace the current texture atlas with a new one.
    ///
    /// If this operation fails, the current texture atlas may be corrupt.
    /// Please load your default texture atlas again if an error ocurred.
    pub fn replace_texture_atlas(
        &mut self,
        textures: &[T],
    ) -> Result<(), atlas::AtlasError> {
        let texture_handles: Vec<texture::Handle> = textures
            .iter()
            .map(|texture| -> texture::Handle {
                let mut handle: texture::Handle = (*texture).into();
                handle.set_hashed_id(texture.hash_to_u64());
                handle
            })
            .collect();

        self.renderer
            .replace_texture_atlas(texture_handles.as_slice())
    }

    pub fn set_background(&mut self, background: Background<T>) {
        self.background = background;
        self.do_full_redraw = true;
    }

    // TODO:
    //   - request_scale_factor(&mut self, scale_factor: f64);
}
