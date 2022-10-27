use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};

pub mod background_layer;
pub mod widget_layer;

pub(crate) use background_layer::BackgroundLayer;
pub(crate) use widget_layer::{WeakRegionTreeEntry, WidgetLayer};

pub use widget_layer::{ContainerRegionRef, ParentAnchorType, RegionInfo};

pub(crate) struct StrongWidgetLayerEntry<MSG> {
    shared: Rc<RefCell<WidgetLayer<MSG>>>,
}

impl<MSG> StrongWidgetLayerEntry<MSG> {
    pub fn new(widget_layer: WidgetLayer<MSG>) -> Self {
        Self {
            shared: Rc::new(RefCell::new(widget_layer)),
        }
    }

    pub fn borrow(&self) -> Ref<'_, WidgetLayer<MSG>> {
        RefCell::borrow(&self.shared)
    }

    pub fn borrow_mut(&mut self) -> RefMut<'_, WidgetLayer<MSG>> {
        RefCell::borrow_mut(&self.shared)
    }

    pub fn downgrade(&self) -> WeakWidgetLayerEntry<MSG> {
        WeakWidgetLayerEntry {
            shared: Rc::downgrade(&self.shared),
        }
    }
}

impl<MSG> Clone for StrongWidgetLayerEntry<MSG> {
    fn clone(&self) -> Self {
        Self {
            shared: Rc::clone(&self.shared),
        }
    }
}

pub(crate) struct WeakWidgetLayerEntry<MSG> {
    shared: Weak<RefCell<WidgetLayer<MSG>>>,
}

impl<MSG> WeakWidgetLayerEntry<MSG> {
    pub fn new() -> Self {
        Self {
            shared: Weak::new(),
        }
    }

    pub fn upgrade(&self) -> Option<StrongWidgetLayerEntry<MSG>> {
        self.shared
            .upgrade()
            .map(|shared| StrongWidgetLayerEntry { shared })
    }
}

impl<MSG> Clone for WeakWidgetLayerEntry<MSG> {
    fn clone(&self) -> Self {
        Self {
            shared: Weak::clone(&self.shared),
        }
    }
}

pub(crate) struct StrongBackgroundLayerEntry {
    shared: Rc<RefCell<BackgroundLayer>>,
}

impl StrongBackgroundLayerEntry {
    pub fn new(layer: BackgroundLayer) -> Self {
        Self {
            shared: Rc::new(RefCell::new(layer)),
        }
    }

    pub fn borrow(&self) -> Ref<'_, BackgroundLayer> {
        RefCell::borrow(&self.shared)
    }

    pub fn borrow_mut(&mut self) -> RefMut<'_, BackgroundLayer> {
        RefCell::borrow_mut(&self.shared)
    }

    pub fn downgrade(&self) -> WeakBackgroundLayerEntry {
        WeakBackgroundLayerEntry {
            shared: Rc::downgrade(&self.shared),
        }
    }
}

impl Clone for StrongBackgroundLayerEntry {
    fn clone(&self) -> Self {
        Self {
            shared: Rc::clone(&self.shared),
        }
    }
}

pub(crate) struct WeakBackgroundLayerEntry {
    shared: Weak<RefCell<BackgroundLayer>>,
}

impl WeakBackgroundLayerEntry {
    pub fn new() -> Self {
        Self {
            shared: Weak::new(),
        }
    }

    pub fn upgrade(&self) -> Option<StrongBackgroundLayerEntry> {
        self.shared
            .upgrade()
            .map(|shared| StrongBackgroundLayerEntry { shared })
    }
}

impl Clone for WeakBackgroundLayerEntry {
    fn clone(&self) -> Self {
        Self {
            shared: Weak::clone(&self.shared),
        }
    }
}

pub struct WidgetLayerRef<MSG> {
    pub(crate) shared: WeakWidgetLayerEntry<MSG>,
}

pub(crate) enum StrongLayerEntry<MSG> {
    Widget(StrongWidgetLayerEntry<MSG>),
    Background(StrongBackgroundLayerEntry),
}
