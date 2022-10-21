use crate::event::{InputEvent, KeyboardEventsListen};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetRegionType {
    /// This widget paints stuff into this region.
    Painted,
    /// This widget does not paint anything into this region,
    /// rather it only uses this region for pointer events.
    PointerOnly,
}

pub trait Widget<MSG> {
    fn on_added(&mut self, msg_out_queue: &mut Vec<MSG>) -> WidgetAddedInfo;

    #[allow(unused)]
    fn on_removed(&mut self, msg_out_queue: &mut Vec<MSG>) {}

    #[allow(unused)]
    fn on_visibility_changed(&mut self, visible: bool, msg_out_queue: &mut Vec<MSG>) {}

    #[allow(unused)]
    fn on_user_message(
        &mut self,
        msg: MSG,
        msg_out_queue: &mut Vec<MSG>,
    ) -> Option<WidgetRequests> {
        None
    }

    fn on_input_event(
        &mut self,
        event: &InputEvent,
        msg_out_queue: &mut Vec<MSG>,
    ) -> EventCapturedStatus;
}

pub struct WidgetAddedInfo {
    pub region_type: WidgetRegionType,
    pub requests: WidgetRequests,
}

pub struct WidgetRequests {
    pub repaint: bool,
    pub set_recieve_next_animation_event: Option<bool>,
    pub set_pointer_events_listen: Option<bool>,
    pub set_keyboard_events_listen: Option<KeyboardEventsListen>,
    pub set_pointer_lock: Option<SetPointerLockType>,
}

impl Default for WidgetRequests {
    fn default() -> Self {
        Self {
            repaint: false,
            set_recieve_next_animation_event: None,
            set_pointer_events_listen: None,
            set_keyboard_events_listen: None,
            set_pointer_lock: None,
        }
    }
}

pub enum EventCapturedStatus {
    NotCaptured,
    Captured(WidgetRequests),
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
