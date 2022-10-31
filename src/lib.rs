mod anchor;
mod app_window;
mod bg_color;
mod layer;
mod node;
mod renderer;

pub(crate) mod widget_node_set;

pub mod error;
pub mod event;
pub mod size;

pub use anchor::{Anchor, HAlign, VAlign};
pub use app_window::AppWindow;
pub use bg_color::{BgColor, GradientDirection};
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

pub fn compute_font_bounds(
    label: &str,
    font_id: femtovg::FontId,
    font_size_pts: f32,
    scale_factor: ScaleFactor,
    vg: &VG,
) -> Size {
    let mut font_paint = femtovg::Paint::color(femtovg::Color::black());
    font_paint.set_font(&[font_id]);
    font_paint.set_font_size(font_size_pts * scale_factor.0);
    font_paint.set_text_baseline(femtovg::Baseline::Middle);

    let font_metrics = vg.measure_text(0.0, 0.0, label, &font_paint).unwrap();

    Size::new(
        font_metrics.width() / scale_factor.0,
        font_metrics.height() / scale_factor.0,
    )
}
