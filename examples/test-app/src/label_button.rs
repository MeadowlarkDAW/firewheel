use crossbeam_channel::Sender;
use firewheel::vg::{Color, FontId, Paint};
use firewheel::GradientDirection;
use firewheel::{
    event::InputEvent, BgColor, EventCapturedStatus, PaintRegionInfo, Point, Rect, ScaleFactor,
    Size, WidgetNode, WidgetNodeRequests, WidgetNodeType, VG,
};
use std::any::Any;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ButtonState {
    Idle,
    KeyboardFocus,
    Hovered,
    Down,
}

pub enum LabelButtonEvent<A: Clone + Send + Sync + 'static> {
    SetLabel(String),
    SetAction {
        action: Option<A>,
        emit_on_release: bool,
    },
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
            hover_bg_color: BgColor::Solid(Color::rgb(71, 71, 71)),
            hover_border_color: Color::rgb(22, 22, 22),
            hover_font_color: Color::rgb(235, 235, 235),

            down_border_width_pts: 1.0,
            down_bg_color: BgColor::Solid(Color::rgb(31, 31, 31)),
            down_border_color: Color::rgb(22, 22, 22),
            down_font_color: Color::rgb(235, 235, 235),

            keyboard_focus_border_width_pts: 1.0,
            keyboard_focus_bg_color: BgColor::Solid(Color::rgb(41, 41, 41)),
            keyboard_focus_border_color: Color::rgb(22, 22, 22),
            keyboard_focus_font_color: Color::rgb(235, 235, 235),
        }
    }
}

pub struct LabelButton<A> {
    label: String,
    font_id: FontId,

    style: Rc<LabelButtonStyle>,

    pointer_bounds: Rect,

    keyboard_focused: bool,

    action: Option<A>,
    emit_on_release: bool,

    state: ButtonState,
}

impl<A: Clone + Send + Sync + 'static> LabelButton<A> {
    pub fn new(
        label: String,
        font_id: FontId,
        style: Rc<LabelButtonStyle>,
        action: Option<A>,
        emit_on_release: bool,
    ) -> Self {
        Self {
            label,
            font_id,
            style,
            pointer_bounds: Rect::default(),
            keyboard_focused: false,
            action,
            emit_on_release,
            state: ButtonState::Idle,
        }
    }
}

impl<A: Clone + Send + Sync + 'static> WidgetNode<A> for LabelButton<A> {
    fn on_added(&mut self, _action_tx: &mut Sender<A>) -> (WidgetNodeType, WidgetNodeRequests) {
        (
            WidgetNodeType::Painted,
            WidgetNodeRequests {
                set_pointer_events_listen: Some(true),
                ..Default::default()
            },
        )
    }

    fn on_visibility_hidden(&mut self, _action_tx: &mut Sender<A>) {
        self.state = ButtonState::Idle;
    }

    fn on_region_changed(&mut self, assigned_rect: Rect) {
        self.pointer_bounds.set_pos(Point::new(
            assigned_rect.x() + f64::from(self.style.margin_lr_pts),
            assigned_rect.y() + f64::from(self.style.margin_tb_pts),
        ));
        self.pointer_bounds.set_size(Size::new(
            (assigned_rect.size().width() - (f32::from(self.style.margin_lr_pts) * 2.0)).max(0.0),
            (assigned_rect.size().height() - (f32::from(self.style.margin_tb_pts) * 2.0)).max(0.0),
        ));
    }

    fn on_user_event(
        &mut self,
        event: Box<dyn Any>,
        _action_tx: &mut Sender<A>,
    ) -> Option<WidgetNodeRequests> {
        if let Ok(event) = event.downcast::<LabelButtonEvent<A>>() {
            match *event {
                LabelButtonEvent::SetLabel(label) => {
                    if self.label != label {
                        self.label = label;

                        return Some(WidgetNodeRequests {
                            repaint: true,
                            ..Default::default()
                        });
                    }
                }
                LabelButtonEvent::SetAction {
                    action,
                    emit_on_release,
                } => {
                    self.action = action;
                    self.emit_on_release = emit_on_release;
                }
                LabelButtonEvent::SetStyle(style) => {
                    self.style = style;

                    return Some(WidgetNodeRequests {
                        repaint: true,
                        ..Default::default()
                    });
                }
                LabelButtonEvent::SetFontID(font_id) => {
                    if self.font_id != font_id {
                        self.font_id = font_id;

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
        action_tx: &mut Sender<A>,
    ) -> EventCapturedStatus {
        match event {
            InputEvent::Pointer(event) => {
                let mouse_is_over = self.pointer_bounds.contains_point(event.position);

                match self.state {
                    ButtonState::Idle | ButtonState::KeyboardFocus => {
                        if mouse_is_over {
                            self.state = ButtonState::Hovered;

                            if event.left_button.just_pressed() {
                                self.state = ButtonState::Down;

                                if !self.emit_on_release {
                                    if let Some(action) = &self.action {
                                        action_tx.send(action.clone()).unwrap();
                                    }
                                }
                            }

                            return EventCapturedStatus::Captured(WidgetNodeRequests {
                                repaint: true,
                                // Listen to when the pointer leaves so we can reset
                                // the state when it does.
                                set_pointer_leave_listen: Some(true),
                                ..Default::default()
                            });
                        }
                    }
                    ButtonState::Hovered => {
                        if mouse_is_over {
                            if event.left_button.just_pressed() {
                                self.state = ButtonState::Down;

                                if !self.emit_on_release {
                                    if let Some(action) = &self.action {
                                        action_tx.send(action.clone()).unwrap();
                                    }
                                }

                                return EventCapturedStatus::Captured(WidgetNodeRequests {
                                    repaint: true,
                                    ..Default::default()
                                });
                            } else {
                                return EventCapturedStatus::Captured(WidgetNodeRequests {
                                    repaint: false,
                                    ..Default::default()
                                });
                            }
                        } else {
                            self.state = if self.keyboard_focused {
                                ButtonState::KeyboardFocus
                            } else {
                                ButtonState::Idle
                            };

                            return EventCapturedStatus::Captured(WidgetNodeRequests {
                                repaint: true,
                                // Stop listening to pointer leave events to free up
                                // resources.
                                set_pointer_leave_listen: Some(false),
                                ..Default::default()
                            });
                        };
                    }
                    ButtonState::Down => {
                        if mouse_is_over {
                            if event.left_button.just_unpressed() {
                                self.state = ButtonState::Hovered;

                                if self.emit_on_release {
                                    if let Some(action) = &self.action {
                                        action_tx.send(action.clone()).unwrap();
                                    }
                                }

                                return EventCapturedStatus::Captured(WidgetNodeRequests {
                                    repaint: true,
                                    ..Default::default()
                                });
                            } else {
                                return EventCapturedStatus::Captured(WidgetNodeRequests {
                                    repaint: false,
                                    ..Default::default()
                                });
                            }
                        } else {
                            self.state = if self.keyboard_focused {
                                ButtonState::KeyboardFocus
                            } else {
                                ButtonState::Idle
                            };

                            return EventCapturedStatus::Captured(WidgetNodeRequests {
                                repaint: true,
                                // Stop listening to pointer leave events to free up
                                // resources.
                                set_pointer_leave_listen: Some(false),
                                ..Default::default()
                            });
                        };
                    }
                }
            }
            _ => {}
        }

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

        let mut bg_path = region.spanning_rounded_rect_path(
            self.style.margin_lr_pts,
            self.style.margin_tb_pts,
            border_width_pts,
            self.style.border_radius_pts,
        );

        let bg_paint = match bg_color {
            BgColor::Solid(color) => Paint::color(*color),
            BgColor::LinearGradient { direction, stops } => match direction {
                GradientDirection::Horizontal => Paint::linear_gradient_stops(
                    0.0,
                    0.0,
                    region.physical_rect.size.width as f32
                        - (f32::from(self.style.margin_lr_pts) * region.scale_factor.0 * 2.0),
                    0.0,
                    stops,
                ),
                GradientDirection::Vertical => Paint::linear_gradient_stops(
                    0.0,
                    0.0,
                    0.0,
                    region.physical_rect.size.height as f32
                        - (f32::from(self.style.margin_tb_pts) * region.scale_factor.0 * 2.0),
                    stops,
                ),
            },
        };

        let mut border_paint = Paint::color(*border_color);
        border_paint.set_line_width((border_width_pts * region.scale_factor.0).round());

        vg.fill_path(&mut bg_path, &bg_paint);
        vg.stroke_path(&mut bg_path, &border_paint);

        let label_rect_width_px = region.physical_rect.size.width as f32
            - (f32::from(self.style.margin_lr_pts + self.style.padding_lr_pts)
                * region.scale_factor.0
                * 2.0)
                .round()
                .max(0.0);
        let label_rect_height_px = self.style.font_size_pts * region.scale_factor.0 * 1.43;

        if label_rect_width_px != 0.0 && label_rect_height_px != 0.0 {
            let label_rect_x_px = region.physical_rect.pos.x as f32
                + (f32::from(self.style.margin_lr_pts + self.style.padding_lr_pts)
                    * region.scale_factor.0);
            let label_rect_y_px = (region.physical_rect.pos.y as f32
                + (region.physical_rect.size.height as f32 / 2.0)
                - (self.style.font_size_pts * 1.43 * region.scale_factor.0 / 2.0))
                .round();

            vg.scissor(
                label_rect_x_px,
                label_rect_y_px,
                label_rect_width_px,
                label_rect_height_px,
            );

            let mut font_paint = Paint::color(*font_color);
            font_paint.set_font(&[self.font_id]);
            font_paint.set_font_size(self.style.font_size_pts * region.scale_factor.0);
            font_paint.set_text_baseline(firewheel::vg::Baseline::Top);

            vg.fill_text(label_rect_x_px, label_rect_y_px, &self.label, &font_paint);

            vg.reset_scissor();
        }
    }
}
