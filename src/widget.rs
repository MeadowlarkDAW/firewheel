use crate::event::{Event, KeyboardEventsListen};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetRegionType {
    /// This widget paints stuff into this region.
    Painted,
    /// This widget does not paint anything into this region,
    /// rather it only uses this region for mouse events.
    MouseOnly,
}

pub trait Widget<MSG> {
    fn on_added(&mut self, msg_out_queue: &mut Vec<MSG>) -> WidgetAddedInfo;

    #[allow(unused)]
    fn on_removed(&mut self, msg_out_queue: &mut Vec<MSG>) {}

    fn on_event(&mut self, event: &Event<MSG>, msg_out_queue: &mut Vec<MSG>)
        -> EventCapturedStatus;
}

pub struct WidgetAddedInfo {
    pub region_type: WidgetRegionType,
    pub recieve_next_animation_event: bool,
    pub listen_to_mouse_events: bool,
    pub keyboard_events_listen: KeyboardEventsListen,
    pub visible: bool,
}

pub struct WidgetRequests {
    pub repaint: bool,
    pub recieve_next_animation_event: bool,
    pub set_mouse_events_listen: Option<bool>,
    pub set_keyboard_events_listen: Option<KeyboardEventsListen>,
    pub set_visibilty: Option<bool>,
    pub lock_mouse_pointer: Option<LockMousePointerType>,
}

impl Default for WidgetRequests {
    fn default() -> Self {
        Self {
            repaint: false,
            recieve_next_animation_event: false,
            set_mouse_events_listen: None,
            set_keyboard_events_listen: None,
            set_visibilty: None,
            lock_mouse_pointer: None,
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
pub enum LockMousePointerType {
    LockToWidget,
    LockInPlaceAndHideCursor,
}
