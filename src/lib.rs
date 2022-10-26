mod anchor;
mod app_window;
mod layer;
mod renderer;

mod widget;

pub mod event;
pub mod size;

pub use anchor::{Anchor, HAlign, VAlign};
pub use app_window::{AppWindow, WidgetRef};
pub use layer::{ContainerRegionRef, LayerError, LayerID, ParentAnchorType, RegionInfo};
pub use size::*;
pub use size::{Point, Rect, ScaleFactor, Size};
pub use widget::{
    EventCapturedStatus, PaintRegionInfo, Widget, WidgetAddedInfo, WidgetRegionType, WidgetRequests,
};

pub use femtovg as vg;
pub type VG = femtovg::Canvas<femtovg::renderer::OpenGl>;
