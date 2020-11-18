use crate::{Bounds, IdGroup, Primitive};

mod tree;
mod v_slider;

pub use tree::Tree;

pub trait Widget {
    type TextureIDs: IdGroup;
    type WidgetIDs: IdGroup;

    fn id(&self) -> Self::WidgetIDs;

    fn needs_redraw(&self) -> bool;

    fn render_bounds(&self) -> Bounds;

    fn primitives<'a>(&self) -> &'a [&'a Primitive<Self::TextureIDs>];
}
