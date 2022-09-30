use smallvec::SmallVec;

use crate::event::{Event, KeyboardEventsListen};
use crate::layer::LayerID;
use crate::ParentAnchorType;
use crate::{Anchor, Point, Size};

pub struct WidgetDrawRegionInfo {
    pub layer: LayerID,
    pub size: Size,
    pub internal_anchor: Anchor,
    pub parent_anchor: Anchor,
    pub parent_anchor_type: ParentAnchorType,
    pub anchor_offset: Point,
}

pub trait Widget<MSG> {
    fn on_added(&mut self) -> WidgetAddedInfo<MSG>;
    fn on_removed(&mut self) -> SmallVec<[Box<MSG>; 4]> {
        SmallVec::new()
    }

    fn on_event(&mut self, event: &Event<MSG>) -> EventCapturedStatus<MSG>;
}

pub struct WidgetAddedInfo<MSG> {
    pub draw_regions: SmallVec<[WidgetDrawRegionInfo; 4]>,
    pub recieve_next_animation_event: bool,
    pub listen_to_mouse_events: bool,
    pub keyboard_events_listen: KeyboardEventsListen,
    pub send_user_events: SmallVec<[Box<MSG>; 4]>,
    pub visible: bool,
}

pub struct WidgetRequests<MSG> {
    pub repaint_regions: SmallVec<[LayerID; 4]>,
    pub recieve_next_animation_event: bool,
    pub set_mouse_events_listen: Option<bool>,
    pub set_keyboard_events_listen: Option<KeyboardEventsListen>,
    pub set_visibilty: Option<bool>,
    pub set_draw_regions: Option<SmallVec<[WidgetDrawRegionInfo; 4]>>,
    pub send_user_events: SmallVec<[Box<MSG>; 4]>,
    pub lock_mouse_pointer: Option<LockMousePointerType>,
}

impl<MSG> Default for WidgetRequests<MSG> {
    fn default() -> Self {
        Self {
            repaint_regions: SmallVec::new(),
            recieve_next_animation_event: false,
            set_mouse_events_listen: None,
            set_keyboard_events_listen: None,
            set_visibilty: None,
            set_draw_regions: None,
            send_user_events: SmallVec::new(),
            lock_mouse_pointer: None,
        }
    }
}

pub enum EventCapturedStatus<MSG> {
    NotCaptured,
    Captured(WidgetRequests<MSG>),
}

impl<MSG> Default for EventCapturedStatus<MSG> {
    fn default() -> Self {
        EventCapturedStatus::NotCaptured
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMousePointerType {
    LockToWidget,
    LockInPlaceAndHideCursor,
}
