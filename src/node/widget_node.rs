use std::any::Any;

use crate::{
    event::{InputEvent, KeyboardEventsListen},
    Rect, VG,
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

pub trait WidgetNode<A: Clone + 'static> {
    fn on_added(&mut self, action_queue: &mut Vec<A>) -> WidgetNodeType;

    #[allow(unused)]
    fn on_visibility_hidden(&mut self, action_queue: &mut Vec<A>) {}

    #[allow(unused)]
    fn on_region_changed(&mut self, assigned_rect: Rect) {}

    #[allow(unused)]
    fn on_user_event(
        &mut self,
        event: Box<dyn Any>,
        action_queue: &mut Vec<A>,
    ) -> Option<WidgetNodeRequests> {
        None
    }

    fn on_input_event(
        &mut self,
        event: &InputEvent,
        action_queue: &mut Vec<A>,
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
    pub set_pointer_leave_listen: Option<bool>,
}

impl Default for WidgetNodeRequests {
    fn default() -> Self {
        Self {
            repaint: false,
            set_recieve_next_animation_event: None,
            set_pointer_events_listen: None,
            set_keyboard_events_listen: None,
            set_pointer_lock: None,
            set_pointer_leave_listen: None,
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
