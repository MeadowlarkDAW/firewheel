mod anchor;
mod app_window;
mod glow_renderer;
mod layer;
mod node;

pub(crate) mod widget_node_set;

pub(crate) use glow_renderer as renderer;

pub mod error;
pub mod event;
pub mod size;

pub use anchor::{Anchor, HAlign, VAlign};
pub use app_window::AppWindow;
pub use error::FirewheelError;
pub use layer::{ContainerRegionRef, ParentAnchorType, RegionInfo};
pub use node::{
    BackgroundNode, EventCapturedStatus, PaintRegionInfo, SetPointerLockType, WidgetNode,
    WidgetNodeRef, WidgetNodeRequests, WidgetNodeType,
};
pub use size::*;
pub use size::{Point, Rect, ScaleFactor, Size};

pub use femtovg as vg;
pub type VG = femtovg::Canvas<femtovg::renderer::OpenGl>;
