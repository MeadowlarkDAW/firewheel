use crate::wgpu_renderer::Renderer;
use crate::{atlas, Background, IdGroup, Texture};
use fnv::FnvHashSet;

pub struct Root<'a, 'b, TexID: IdGroup> {
    window: &'a mut baseview::Window<'b>,
    renderer: &'a mut Renderer<TexID>,
}

impl<'a, 'b, TexID: IdGroup> Root<'a, 'b, TexID> {
    pub(crate) fn new(
        window: &'a mut baseview::Window<'b>,
        renderer: &'a mut Renderer<TexID>,
    ) -> Self {
        Self { window, renderer }
    }

    /// Replace the current texture atlas with a new one.
    ///
    /// If this operation fails, the current texture atlas may be corrupt.
    /// Please load your default texture atlas again if an error ocurred.
    pub fn replace_texture_atlas(
        &mut self,
        textures: &[(TexID, Texture)],
    ) -> Result<(), atlas::AtlasError> {
        // check for duplicate ids
        {
            let mut ids = FnvHashSet::default();
            for (id, _) in textures {
                if !ids.insert(*id) {
                    return Err(atlas::AtlasError::IdNotUnique(format!(
                        "{:?}",
                        *id
                    )));
                }
            }
        }

        self.renderer.replace_texture_atlas(textures)
    }

    /// Set the window background from one or multiple multiple backgrounds.
    pub fn set_background(&mut self, background: Background<TexID>) {
        self.renderer.set_background(background);
    }

    // TODO:
    //   - request_scale_factor(&mut self, scale_factor: f64);
}
