use crate::wgpu_renderer::Renderer;
use crate::widgets::WidgetTree;
use crate::{atlas, Background, IdGroup, Size, Texture};
use baseview::Window;
use std::collections::HashSet;

pub(crate) struct RootState<TexID: IdGroup, WidgetID: IdGroup> {
    widget_tree: WidgetTree<TexID, WidgetID>,
}

impl<TexID: IdGroup, WidgetID: IdGroup> RootState<TexID, WidgetID> {
    pub fn new() -> Self {
        Self {
            widget_tree: WidgetTree::new(),
        }
    }
}

pub struct Root<'a, TexID: IdGroup, WidgetID: IdGroup> {
    state: &'a mut RootState<TexID, WidgetID>,
    window: &'a mut baseview::Window,
    renderer: &'a mut Renderer,
}

impl<'a, TexID: IdGroup, WidgetID: IdGroup> Root<'a, TexID, WidgetID> {
    pub(crate) fn new(
        state: &'a mut RootState<TexID, WidgetID>,
        window: &'a mut baseview::Window,
        renderer: &'a mut Renderer,
    ) -> Self {
        Self {
            state,
            window,
            renderer,
        }
    }

    /// Replace the current texture atlas with a new one.
    ///
    /// If this operation fails, the current texture atlas may be corrupt.
    /// Please load your default texture atlas again if an error ocurred.
    pub fn replace_texture_atlas(
        &mut self,
        textures: &[(TexID, Texture)],
    ) -> Result<(), atlas::AtlasError>
    where
        TexID: crate::IdGroup,
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
                (id.hash_to_u64(), texture)
            })
            .collect();

        self.renderer.replace_texture_atlas(textures.as_slice())
    }

    /// Set the window background from one or multiple multiple backgrounds.
    pub fn set_background(&mut self, background: Background<TexID>) {
        self.renderer.set_background(background.hash_to_u64());
    }

    // TODO:
    //   - request_scale_factor(&mut self, scale_factor: f64);
}
