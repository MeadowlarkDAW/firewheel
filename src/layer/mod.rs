use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};

pub mod background_layer;
pub mod widget_layer;

pub(crate) use background_layer::BackgroundLayer;
pub(crate) use widget_layer::{WeakRegionTreeEntry, WidgetLayer};

pub use widget_layer::{ContainerRegionRef, ParentAnchorType, RegionInfo};

pub(crate) struct StrongWidgetLayerEntry<A: Clone + 'static> {
    shared: Rc<RefCell<WidgetLayer<A>>>,
}

impl<A: Clone + 'static> StrongWidgetLayerEntry<A> {
    pub fn new(widget_layer: WidgetLayer<A>) -> Self {
        Self {
            shared: Rc::new(RefCell::new(widget_layer)),
        }
    }

    pub fn borrow(&self) -> Ref<'_, WidgetLayer<A>> {
        RefCell::borrow(&self.shared)
    }

    pub fn borrow_mut(&mut self) -> RefMut<'_, WidgetLayer<A>> {
        RefCell::borrow_mut(&self.shared)
    }

    pub fn downgrade(&self) -> WeakWidgetLayerEntry<A> {
        WeakWidgetLayerEntry {
            shared: Rc::downgrade(&self.shared),
        }
    }
}

impl<A: Clone + 'static> Clone for StrongWidgetLayerEntry<A> {
    fn clone(&self) -> Self {
        Self {
            shared: Rc::clone(&self.shared),
        }
    }
}

pub(crate) struct WeakWidgetLayerEntry<A: Clone + 'static> {
    shared: Weak<RefCell<WidgetLayer<A>>>,
}

impl<A: Clone + 'static> WeakWidgetLayerEntry<A> {
    pub fn new() -> Self {
        Self {
            shared: Weak::new(),
        }
    }

    pub fn upgrade(&self) -> Option<StrongWidgetLayerEntry<A>> {
        self.shared
            .upgrade()
            .map(|shared| StrongWidgetLayerEntry { shared })
    }
}

impl<A: Clone + 'static> Clone for WeakWidgetLayerEntry<A> {
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

pub struct WidgetLayerRef<A: Clone + 'static> {
    pub(crate) shared: WeakWidgetLayerEntry<A>,
}

pub(crate) enum StrongLayerEntry<A: Clone + 'static> {
    Widget(StrongWidgetLayerEntry<A>),
    Background(StrongBackgroundLayerEntry),
}
