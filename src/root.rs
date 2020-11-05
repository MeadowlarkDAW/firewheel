use crate::wgpu_renderer::Renderer;
use crate::{atlas, texture, Background, Color, Size, Texture, Window};
use futures::executor::block_on;

pub struct Root {
    pub(crate) renderer: Renderer,
    background: Background,
    do_full_redraw: bool,
}

impl Root {
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
            self.renderer.render(self.do_full_redraw, &self.background);

            self.do_full_redraw = false;
        }
    }

    /// Replace the current texture atlas with a new one.
    ///
    /// If this operation fails, the current texture atlas may be corrupt.
    /// Please load your default texture atlas again if an error ocurred.
    pub fn replace_texture_atlas<TexID: std::hash::Hash + Copy + Clone>(
        &mut self,
        textures: &[(TexID, Texture)],
    ) -> Result<(), atlas::AtlasError> {
        let textures: Vec<(u64, &Texture)> = textures
            .iter()
            .map(|(id, texture)| -> (u64, &Texture) {
                (crate::hash_id(id), texture)
            })
            .collect();

        self.renderer.replace_texture_atlas(textures.as_slice())
    }

    pub fn set_background(&mut self, background: Background) {
        self.background = background;
        self.do_full_redraw = true;
    }

    // TODO:
    //   - request_scale_factor(&mut self, scale_factor: f64);
}
