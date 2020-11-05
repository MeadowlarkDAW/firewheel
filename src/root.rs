use crate::wgpu_renderer::Renderer;
use crate::{atlas, Background, Color, Size, Texture};
use baseview::Window;
use futures::executor::block_on;

pub(crate) struct RootState {
    pub renderer: Renderer,
    background: Background,
    do_full_redraw: bool,
}

impl RootState {
    pub fn new(window: &mut Window) -> Self {
        let physical_size = Size::<u16>::new(
            window.window_info().physical_size().width as u16,
            window.window_info().physical_size().height as u16,
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

    pub fn window_resized(&mut self, physical_size: Size<u16>, scale: f64) {
        self.renderer.resize(physical_size, scale);
        self.do_full_redraw = true;
    }

    pub fn render(&mut self) {
        if self.do_full_redraw {
            self.renderer.render(self.do_full_redraw, &self.background);

            self.do_full_redraw = false;
        }
    }
}

pub struct Root<'a> {
    state: &'a mut RootState,
    window: &'a mut baseview::Window,
}

impl<'a> Root<'a> {
    pub(crate) fn new(
        state: &'a mut RootState,
        window: &'a mut baseview::Window,
    ) -> Self {
        Self { state, window }
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

        self.state
            .renderer
            .replace_texture_atlas(textures.as_slice())
    }

    /// Set the window background from one or multiple multiple backgrounds.
    pub fn set_background(&mut self, background: Background) {
        self.state.background = background;
        self.state.do_full_redraw = true;
    }

    pub fn fit_window_to_background(&mut self) {
        // TODO: request window size
    }

    // TODO:
    //   - request_scale_factor(&mut self, scale_factor: f64);
}
