mod anchor;
mod canvas;
mod layer;
mod renderer;

mod widget;

pub mod event;
pub mod size;

pub use anchor::{Anchor, HAlign, VAlign};
pub use canvas::{Canvas, WidgetRef};
pub use layer::{ContainerRegionID, LayerError, LayerID, ParentAnchorType, RegionInfo};
pub use size::{Point, Rect, ScaleFactor, Size};
pub use widget::{
    EventCapturedStatus, PaintRegionInfo, Widget, WidgetAddedInfo, WidgetRegionType, WidgetRequests,
};

pub type VG = femtovg::Canvas<femtovg::renderer::OpenGl>;
