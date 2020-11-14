use crate::wgpu_renderer::Renderer;
use crate::{atlas, Background, Color, Size, Texture};
use baseview::Window;
use futures::executor::block_on;
use std::collections::HashSet;

pub(crate) struct RootState {
    pub renderer: Renderer,
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

        Self { renderer }
    }

    pub fn window_resized(&mut self, physical_size: Size<u16>, scale: f64) {
        self.renderer.resize(physical_size, scale);
    }

    pub fn render(&mut self) {
        self.renderer.render();
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
    pub fn replace_texture_atlas<TexID>(
        &mut self,
        textures: &[(TexID, Texture)],
    ) -> Result<(), atlas::AtlasError>
    where
        TexID:
            std::hash::Hash + std::fmt::Debug + Eq + PartialEq + Copy + Clone,
    {
        // check for duplicate ids
        {
            let mut ids = HashSet::new();
            for (id, _) in textures {
                if !ids.insert(*id) {
                    return Err(atlas::AtlasError::IdNotUnique(format!(
                        "{:?}",
                        *id
                    )));
                }
            }
        }

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
        self.state.renderer.set_background(background);
    }

    // TODO:
    //   - request_scale_factor(&mut self, scale_factor: f64);
}
