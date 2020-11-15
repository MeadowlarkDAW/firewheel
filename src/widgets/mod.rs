use crate::{Bounds, IdGroup};

mod tree;
mod v_slider;

pub(crate) use tree::WidgetTree;

pub trait Widget {
    type TextureIDs: IdGroup;

    fn render_bounds(&self) -> Bounds;

    fn state_changed(&self) -> bool;
}
