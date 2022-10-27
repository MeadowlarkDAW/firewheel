use std::cell::{RefCell, RefMut};
use std::hash::Hash;
use std::rc::Rc;

use crate::layer::{WeakBackgroundLayerEntry, WeakRegionTreeEntry, WeakWidgetLayerEntry};
use crate::size::{PhysicalRect, Rect, ScaleFactor};

mod background_node;
mod widget_node;
pub use background_node::BackgroundNode;
pub use widget_node::{
    EventCapturedStatus, SetPointerLockType, WidgetNode, WidgetNodeRequests, WidgetNodeType,
};

#[derive(Debug, Clone, Copy)]
pub struct PaintRegionInfo {
    /// This widget's assigned rectangular region in logical coordinates.
    pub rect: Rect,

    /// The layer's visible rectangular region in logical coordinates.
    pub layer_rect: Rect,

    /// This widget's assigned rectangular region in physical coordinates
    /// (the physical coordinates in the layer's texture, not the screen).
    pub physical_rect: PhysicalRect,

    /// The layer's visible rectangular region in physical coordinates
    /// (the physical coordinates in the layer's texture, not the screen).
    pub layer_physical_rect: PhysicalRect,

    /// The dpi scaling factor.
    pub scale_factor: ScaleFactor,
}

pub(crate) struct StrongWidgetNodeEntry<MSG> {
    shared: Rc<RefCell<Box<dyn WidgetNode<MSG>>>>,
    assigned_layer: WeakWidgetLayerEntry<MSG>,
    assigned_region: WeakRegionTreeEntry<MSG>,
    unique_id: u64,
}

impl<MSG> StrongWidgetNodeEntry<MSG> {
    pub fn new(
        shared: Rc<RefCell<Box<dyn WidgetNode<MSG>>>>,
        assigned_layer: WeakWidgetLayerEntry<MSG>,
        assigned_region: WeakRegionTreeEntry<MSG>,
        unique_id: u64,
    ) -> Self {
        Self {
            shared,
            assigned_layer,
            assigned_region,
            unique_id,
        }
    }

    pub fn borrow_mut(&mut self) -> RefMut<'_, Box<dyn WidgetNode<MSG>>> {
        RefCell::borrow_mut(&self.shared)
    }

    pub fn unique_id(&self) -> u64 {
        self.unique_id
    }

    pub fn set_assigned_region(&mut self, region: WeakRegionTreeEntry<MSG>) {
        self.assigned_region = region;
    }

    pub fn assigned_layer_mut(&mut self) -> &mut WeakWidgetLayerEntry<MSG> {
        &mut self.assigned_layer
    }

    pub fn assigned_region(&self) -> &WeakRegionTreeEntry<MSG> {
        &self.assigned_region
    }

    pub fn assigned_region_mut(&mut self) -> &mut WeakRegionTreeEntry<MSG> {
        &mut self.assigned_region
    }
}

impl<MSG> Clone for StrongWidgetNodeEntry<MSG> {
    fn clone(&self) -> Self {
        Self {
            shared: Rc::clone(&self.shared),
            assigned_layer: self.assigned_layer.clone(),
            assigned_region: self.assigned_region.clone(),
            unique_id: self.unique_id,
        }
    }
}

impl<MSG> PartialEq for StrongWidgetNodeEntry<MSG> {
    fn eq(&self, other: &Self) -> bool {
        self.unique_id.eq(&other.unique_id)
    }
}

impl<MSG> Eq for StrongWidgetNodeEntry<MSG> {}

impl<MSG> Hash for StrongWidgetNodeEntry<MSG> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.unique_id.hash(state)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct WidgetNodeRef<MSG> {
    pub(crate) shared: StrongWidgetNodeEntry<MSG>,
}

impl<MSG> WidgetNodeRef<MSG> {
    pub fn unique_id(&self) -> u64 {
        self.shared.unique_id
    }
}

pub(crate) struct StrongBackgroundNodeEntry {
    shared: Rc<RefCell<Box<dyn BackgroundNode>>>,
    assigned_layer: WeakBackgroundLayerEntry,
    unique_id: u64,
}

impl StrongBackgroundNodeEntry {
    // Used by the unit tests.
    #[allow(unused)]
    pub fn new(background_node: Box<dyn BackgroundNode>, unique_id: u64) -> Self {
        Self {
            shared: Rc::new(RefCell::new(background_node)),
            assigned_layer: WeakBackgroundLayerEntry::new(),
            unique_id,
        }
    }

    pub fn set_assigned_layer(&mut self, layer: WeakBackgroundLayerEntry) {
        self.assigned_layer = layer;
    }

    pub fn assigned_layer_mut(&mut self) -> &mut WeakBackgroundLayerEntry {
        &mut self.assigned_layer
    }

    pub fn borrow_mut(&mut self) -> RefMut<'_, Box<dyn BackgroundNode>> {
        RefCell::borrow_mut(&self.shared)
    }
}

impl Clone for StrongBackgroundNodeEntry {
    fn clone(&self) -> Self {
        Self {
            shared: Rc::clone(&self.shared),
            assigned_layer: self.assigned_layer.clone(),
            unique_id: self.unique_id,
        }
    }
}

impl PartialEq for StrongBackgroundNodeEntry {
    fn eq(&self, other: &Self) -> bool {
        self.unique_id.eq(&other.unique_id)
    }
}

impl Eq for StrongBackgroundNodeEntry {}

impl Hash for StrongBackgroundNodeEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.unique_id.hash(state)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct BackgroundNodeRef {
    pub(crate) shared: StrongBackgroundNodeEntry,
}

impl BackgroundNodeRef {
    pub fn unique_id(&self) -> u64 {
        self.shared.unique_id
    }
}
