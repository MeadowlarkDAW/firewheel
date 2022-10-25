mod anchor;
mod layer;
mod renderer;
mod window_canvas;

mod widget;

pub mod event;
pub mod size;

pub use anchor::{Anchor, HAlign, VAlign};
pub use layer::{ContainerRegionID, LayerError, LayerID, ParentAnchorType, RegionInfo};
pub use size::*;
pub use size::{Point, Rect, ScaleFactor, Size};
pub use widget::{
    EventCapturedStatus, PaintRegionInfo, Widget, WidgetAddedInfo, WidgetRegionType, WidgetRequests,
};
pub use window_canvas::{WidgetRef, WindowCanvas};

pub use femtovg as vg;
pub type VG = femtovg::Canvas<femtovg::renderer::OpenGl>;
