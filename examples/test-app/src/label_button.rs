use firewheel::vg::{Color, FontId, Paint};
use firewheel::{
    event::InputEvent, BgColor, EventCapturedStatus, PaintRegionInfo, Rect, ScaleFactor, Size,
    WidgetNode, WidgetNodeRequests, WidgetNodeType, VG,
};
use std::any::Any;
use std::rc::Rc;

enum ButtonState {
    Idle,
    Hovered,
    Down,
    KeyboardFocus,
}

pub enum LabelButtonEvent {
    SetLabel(String),
    SetStyle(Rc<LabelButtonStyle>),
    SetFontID(FontId),
}

#[derive(Debug, Clone)]
pub struct LabelButtonStyle {
    pub padding_lr_pts: u16,
    pub padding_tb_pts: u16,
    pub margin_lr_pts: u16,
    pub margin_tb_pts: u16,

    pub font_size_pts: f32,

    pub border_radius_pts: f32,

    pub idle_border_width_pts: f32,
    pub idle_bg_color: BgColor,
    pub idle_border_color: Color,
    pub idle_font_color: Color,

    pub hover_border_width_pts: f32,
    pub hover_bg_color: BgColor,
    pub hover_border_color: Color,
    pub hover_font_color: Color,

    pub down_border_width_pts: f32,
    pub down_bg_color: BgColor,
    pub down_border_color: Color,
    pub down_font_color: Color,

    pub keyboard_focus_border_width_pts: f32,
    pub keyboard_focus_bg_color: BgColor,
    pub keyboard_focus_border_color: Color,
    pub keyboard_focus_font_color: Color,
}

impl LabelButtonStyle {
    pub fn compute_size(
        &self,
        label: &str,
        font_id: FontId,
        scale_factor: ScaleFactor,
        vg: &VG,
    ) -> Size {
        let font_bounds_pts =
            firewheel::compute_font_bounds(label, font_id, self.font_size_pts, scale_factor, vg);

        let full_width_pts = (font_bounds_pts.width()
            + (f32::from(self.padding_lr_pts + self.margin_lr_pts) * 2.0))
            .ceil();
        let full_height_pts = (font_bounds_pts.height()
            + (f32::from(self.padding_tb_pts + self.margin_tb_pts) * 2.0))
            .ceil();

        Size::new(full_width_pts, full_height_pts)
    }
}

impl Default for LabelButtonStyle {
    fn default() -> Self {
        Self {
            padding_lr_pts: 8,
            padding_tb_pts: 8,
            margin_lr_pts: 0,
            margin_tb_pts: 0,

            font_size_pts: 16.0,

            border_radius_pts: 3.0,

            idle_border_width_pts: 1.0,
            idle_bg_color: BgColor::Solid(Color::rgb(41, 41, 41)),
            idle_border_color: Color::rgb(22, 22, 22),
            idle_font_color: Color::rgb(235, 235, 235),

            hover_border_width_pts: 1.0,
            hover_bg_color: BgColor::Solid(Color::rgb(41, 41, 41)),
            hover_border_color: Color::rgb(22, 22, 22),
            hover_font_color: Color::rgb(235, 235, 235),

            down_border_width_pts: 1.0,
            down_bg_color: BgColor::Solid(Color::rgb(41, 41, 41)),
            down_border_color: Color::rgb(22, 22, 22),
            down_font_color: Color::rgb(235, 235, 235),

            keyboard_focus_border_width_pts: 1.0,
            keyboard_focus_bg_color: BgColor::Solid(Color::rgb(41, 41, 41)),
            keyboard_focus_border_color: Color::rgb(22, 22, 22),
            keyboard_focus_font_color: Color::rgb(235, 235, 235),
        }
    }
}

pub struct LabelButton {
    label: String,
    font_id: FontId,

    style: Rc<LabelButtonStyle>,

    font_bounds_pts: Size,
    do_recompute_font_bounds: bool,

    state: ButtonState,
}

impl LabelButton {
    pub fn new(label: String, font_id: FontId, style: Rc<LabelButtonStyle>) -> Self {
        Self {
            label,
            font_id,
            style,
            font_bounds_pts: Size::default(),
            do_recompute_font_bounds: true,
            state: ButtonState::Idle,
        }
    }
}

impl<MSG> WidgetNode<MSG> for LabelButton {
    fn on_added(&mut self, _msg_out_queue: &mut Vec<MSG>) -> WidgetNodeType {
        WidgetNodeType::Painted
    }

    fn on_visibility_hidden(&mut self, _msg_out_queue: &mut Vec<MSG>) {
        //println!("button hidden");
    }

    fn on_region_changed(&mut self, _assigned_rect: Rect) {
        //println!("region changed");
    }

    fn on_user_event(
        &mut self,
        event: Box<dyn Any>,
        _msg_out_queue: &mut Vec<MSG>,
    ) -> Option<WidgetNodeRequests> {
        if let Some(event) = event.downcast_ref::<LabelButtonEvent>() {
            match event {
                LabelButtonEvent::SetLabel(label) => {
                    if &self.label != label {
                        self.label = label.clone();
                        self.do_recompute_font_bounds = true;

                        return Some(WidgetNodeRequests {
                            repaint: true,
                            ..Default::default()
                        });
                    }
                }
                LabelButtonEvent::SetStyle(style) => {
                    if style.font_size_pts != self.style.font_size_pts {
                        self.do_recompute_font_bounds = true;
                    }
                    self.style = style.clone();

                    return Some(WidgetNodeRequests {
                        repaint: true,
                        ..Default::default()
                    });
                }
                LabelButtonEvent::SetFontID(font_id) => {
                    if self.font_id != *font_id {
                        self.font_id = *font_id;
                        self.do_recompute_font_bounds = true;

                        return Some(WidgetNodeRequests {
                            repaint: true,
                            ..Default::default()
                        });
                    }
                }
            }
        }

        None
    }

    fn on_input_event(
        &mut self,
        event: &InputEvent,
        msg_out_queue: &mut Vec<MSG>,
    ) -> EventCapturedStatus {
        EventCapturedStatus::NotCaptured
    }

    #[allow(unused)]
    fn paint(&mut self, vg: &mut VG, region: &PaintRegionInfo) {
        let (border_width_pts, bg_color, border_color, font_color) = match self.state {
            ButtonState::Idle => (
                self.style.idle_border_width_pts,
                &self.style.idle_bg_color,
                &self.style.idle_border_color,
                &self.style.idle_font_color,
            ),
            ButtonState::Hovered => (
                self.style.hover_border_width_pts,
                &self.style.hover_bg_color,
                &self.style.hover_border_color,
                &self.style.hover_font_color,
            ),
            ButtonState::Down => (
                self.style.down_border_width_pts,
                &self.style.down_bg_color,
                &self.style.down_border_color,
                &self.style.down_font_color,
            ),
            ButtonState::KeyboardFocus => (
                self.style.keyboard_focus_border_width_pts,
                &self.style.keyboard_focus_bg_color,
                &self.style.keyboard_focus_border_color,
                &self.style.keyboard_focus_font_color,
            ),
        };

        if self.do_recompute_font_bounds {
            self.do_recompute_font_bounds = false;

            self.font_bounds_pts = firewheel::compute_font_bounds(
                &self.label,
                self.font_id,
                self.style.font_size_pts,
                region.scale_factor,
                vg,
            );
        }

        let mut bg_path = region.spanning_rounded_rect_path(
            self.style.margin_lr_pts,
            self.style.margin_tb_pts,
            border_width_pts,
            self.style.border_radius_pts,
        );

        let mut bg_paint = Paint::color(Color::rgb(41, 41, 41));
        let mut bg_stroke_paint = Paint::color(Color::rgb(22, 22, 22));
        bg_stroke_paint.set_line_width((border_width_pts * region.scale_factor.0).round());

        vg.fill_path(&mut bg_path, &bg_paint);
        vg.stroke_path(&mut bg_path, &bg_stroke_paint);

        let label_rect_width_px = region.physical_rect.size.width as f32
            - (f32::from(self.style.margin_lr_pts + self.style.padding_lr_pts)
                * region.scale_factor.0
                * 2.0)
                .round()
                .max(0.0);
        let label_rect_height_px = region.physical_rect.size.height as f32
            - (f32::from(self.style.margin_tb_pts + self.style.padding_tb_pts)
                * region.scale_factor.0
                * 2.0)
                .round()
                .max(0.0);

        if label_rect_width_px != 0.0 && label_rect_height_px != 0.0 {
            let label_rect_x_px = region.physical_rect.pos.x as f32
                + (f32::from(self.style.margin_lr_pts + self.style.padding_lr_pts)
                    * region.scale_factor.0);
            let label_rect_y_px = region.physical_rect.pos.y as f32
                + (f32::from(self.style.margin_tb_pts + self.style.padding_tb_pts)
                    * region.scale_factor.0);
            let label_rect_y_center_px = (region.physical_rect.pos.y as f32
                + (f32::from(self.style.margin_tb_pts + self.style.padding_tb_pts)
                    * region.scale_factor.0)
                + (self.font_bounds_pts.height() * region.scale_factor.0 / 2.0))
                .round()
                - 0.5;

            vg.scissor(
                label_rect_x_px,
                label_rect_y_px,
                label_rect_width_px,
                label_rect_height_px,
            );

            let mut font_paint = Paint::color(*font_color);
            font_paint.set_font(&[self.font_id]);
            font_paint.set_font_size(self.style.font_size_pts * region.scale_factor.0);
            font_paint.set_text_baseline(firewheel::vg::Baseline::Middle);

            vg.fill_text(
                label_rect_x_px,
                label_rect_y_center_px,
                &self.label,
                &font_paint,
            );

            vg.reset_scissor();
        }
    }
}
