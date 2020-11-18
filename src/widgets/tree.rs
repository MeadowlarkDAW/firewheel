use super::Widget;
use crate::{Bounds, IdGroup, Primitive};
use fnv::FnvHashMap;

pub struct RenderUpdateInfo<'a, TexID: IdGroup> {
    pub render_areas: Vec<Bounds>,
    pub render_primitives: Vec<&'a Primitive<TexID>>,
}

pub struct Tree<TexID: IdGroup, WidgetID: IdGroup> {
    widgets: FnvHashMap<
        WidgetID,
        &'static mut dyn Widget<TextureIDs = TexID, WidgetIDs = WidgetID>,
    >,
    previous_bounds: FnvHashMap<WidgetID, Bounds>,
}

impl<TexID: IdGroup, WidgetID: IdGroup> Tree<TexID, WidgetID> {
    pub(crate) fn new() -> Self {
        Self {
            widgets: FnvHashMap::default(),
            previous_bounds: FnvHashMap::default(),
        }
    }

    pub fn add(
        &mut self,
        widget: &'static mut impl Widget<TextureIDs = TexID, WidgetIDs = WidgetID>,
    ) {
        let id = widget.id();
        if let Some(_) = self.widgets.insert(id, widget) {
            panic!("Two widgets added with same ID {:?}", id);
        }
    }

    pub(crate) fn start_tree_construction(&mut self) {
        self.widgets.clear();
    }

    pub(crate) fn get_render_info<'a>(
        &mut self,
        do_full_redraw: bool,
    ) -> RenderUpdateInfo<'a, TexID> {
        let mut render_areas: Vec<Bounds> =
            Vec::with_capacity(self.widgets.len());
        let mut render_primitives: Vec<&Primitive<TexID>> =
            Vec::with_capacity(self.widgets.len() * 2);

        if do_full_redraw {
            // Clear previous render bounds and allocate new space if necessary.
            self.previous_bounds.clear();
            if self.previous_bounds.len() < self.widgets.len() {
                self.previous_bounds
                    .reserve(self.widgets.len() - self.previous_bounds.len())
            }

            // Render all widgets. No need for render areas because the whole screen is being redrawn.
            for (id, widget) in &self.widgets {
                let _ =
                    self.previous_bounds.insert(*id, widget.render_bounds());
                render_primitives.extend_from_slice(widget.primitives());
            }
        } else {
            let mut new_bounds: FnvHashMap<WidgetID, Bounds> =
                FnvHashMap::with_capacity_and_hasher(
                    self.widgets.len(),
                    Default::default(),
                );

            for (id, widget) in &self.widgets {
                if let Some(previous_bounds) = self.previous_bounds.get(&id) {
                    if widget.needs_redraw() {
                        let widget_bounds = widget.render_bounds();

                        if widget_bounds != *previous_bounds {
                            // Widget has changed location/size. Redraw the area where it used to be.
                            render_areas.push(*previous_bounds);
                        }

                        // Redraw the widget.
                        render_areas.push(widget_bounds);
                        render_primitives
                            .extend_from_slice(widget.primitives());

                        let _ = new_bounds.insert(*id, widget_bounds);
                    } else {
                        // Widget has not changed, use the previous bounds.
                        let _ = new_bounds.insert(*id, *previous_bounds);
                    }
                } else {
                    // Widget has not existed previously, so draw it.
                    let widget_bounds = widget.render_bounds();

                    render_areas.push(widget_bounds);
                    render_primitives.extend_from_slice(widget.primitives());

                    let _ = new_bounds.insert(*id, widget_bounds);
                }
            }

            // Store the bounds of the existing widgets in the tree, while removing those
            // that do not exist anymore.
            self.previous_bounds = new_bounds;
        }

        RenderUpdateInfo {
            render_areas,
            render_primitives,
        }
    }
}