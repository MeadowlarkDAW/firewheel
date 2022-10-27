use std::any::Any;

use crate::{
    event::{InputEvent, KeyboardEventsListen},
    VG,
};

use super::PaintRegionInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetNodeType {
    /// This widget paints stuff into this region.
    Painted,
    /// This widget does not paint anything into this region,
    /// rather it only uses this region for pointer events.
    PointerOnly,
}

pub trait WidgetNode<MSG> {
    fn on_added(&mut self, msg_out_queue: &mut Vec<MSG>) -> WidgetNodeType;

    #[allow(unused)]
    fn on_removed(&mut self, msg_out_queue: &mut Vec<MSG>) {}

    #[allow(unused)]
    fn on_visibility_hidden(&mut self, msg_out_queue: &mut Vec<MSG>) {}

    #[allow(unused)]
    fn on_user_event(
        &mut self,
        event: Box<dyn Any>,
        msg_out_queue: &mut Vec<MSG>,
    ) -> Option<WidgetNodeRequests> {
        None
    }

    fn on_input_event(
        &mut self,
        event: &InputEvent,
        msg_out_queue: &mut Vec<MSG>,
    ) -> EventCapturedStatus;

    #[allow(unused)]
    fn paint(&mut self, vg: &mut VG, region: &PaintRegionInfo) {}
}

pub struct WidgetNodeRequests {
    pub repaint: bool,
    pub set_recieve_next_animation_event: Option<bool>,
    pub set_pointer_events_listen: Option<bool>,
    pub set_keyboard_events_listen: Option<KeyboardEventsListen>,
    pub set_pointer_lock: Option<SetPointerLockType>,
    pub set_pointer_down_listen: Option<bool>,
}

impl Default for WidgetNodeRequests {
    fn default() -> Self {
        Self {
            repaint: false,
            set_recieve_next_animation_event: None,
            set_pointer_events_listen: None,
            set_keyboard_events_listen: None,
            set_pointer_lock: None,
            set_pointer_down_listen: None,
        }
    }
}

pub enum EventCapturedStatus {
    NotCaptured,
    Captured(WidgetNodeRequests),
}

impl Default for EventCapturedStatus {
    fn default() -> Self {
        EventCapturedStatus::NotCaptured
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetPointerLockType {
    Unlock,
    LockToWidget,
    LockInPlaceAndHideCursor,
}
