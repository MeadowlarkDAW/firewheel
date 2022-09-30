use fnv::{FnvHashMap, FnvHashSet};
use std::cell::{Ref, RefCell, RefMut};
use std::hash::Hash;
use std::rc::{Rc, Weak};

use crate::anchor::Anchor;
use crate::event::KeyboardEventsListen;
use crate::layer::{Layer, LayerError, LayerID};
use crate::widget::{LockMousePointerType, Widget};
use crate::{ContainerRegionID, ParentAnchorType, Point, RegionInfo, Size};

struct StrongLayerEntry<MSG> {
    shared: Rc<RefCell<Layer<MSG>>>,
}

impl<MSG> StrongLayerEntry<MSG> {
    fn borrow(&self) -> Ref<'_, Layer<MSG>> {
        RefCell::borrow(&self.shared)
    }

    fn borrow_mut(&mut self) -> RefMut<'_, Layer<MSG>> {
        RefCell::borrow_mut(&self.shared)
    }

    fn downgrade(&self) -> WeakLayerEntry<MSG> {
        WeakLayerEntry {
            shared: Rc::downgrade(&self.shared),
        }
    }
}

impl<MSG> Clone for StrongLayerEntry<MSG> {
    fn clone(&self) -> Self {
        Self {
            shared: Rc::clone(&self.shared),
        }
    }
}

pub(crate) struct WeakLayerEntry<MSG> {
    shared: Weak<RefCell<Layer<MSG>>>,
}

impl<MSG> WeakLayerEntry<MSG> {
    fn upgrade(&self) -> Option<StrongLayerEntry<MSG>> {
        self.shared
            .upgrade()
            .map(|shared| StrongLayerEntry { shared })
    }
}

impl<MSG> Clone for WeakLayerEntry<MSG> {
    fn clone(&self) -> Self {
        Self {
            shared: Weak::clone(&self.shared),
        }
    }
}

pub(crate) struct StrongWidgetEntry<MSG> {
    shared: Rc<RefCell<Box<dyn Widget<MSG>>>>,
    assigned_layer: WeakLayerEntry<MSG>,
    unique_id: u64,
}

impl<MSG> StrongWidgetEntry<MSG> {
    /*
    pub fn borrow(&self) -> Ref<'_, Box<dyn Widget<MSG>>> {
        RefCell::borrow(&self.shared)
    }
    */

    pub fn borrow_mut(&mut self) -> RefMut<'_, Box<dyn Widget<MSG>>> {
        RefCell::borrow_mut(&self.shared)
    }

    pub fn unique_id(&self) -> u64 {
        self.unique_id
    }
}

impl<MSG> Clone for StrongWidgetEntry<MSG> {
    fn clone(&self) -> Self {
        Self {
            shared: Rc::clone(&self.shared),
            assigned_layer: self.assigned_layer.clone(),
            unique_id: self.unique_id,
        }
    }
}

impl<MSG> PartialEq for StrongWidgetEntry<MSG> {
    fn eq(&self, other: &Self) -> bool {
        self.unique_id.eq(&other.unique_id)
    }
}

impl<MSG> Eq for StrongWidgetEntry<MSG> {}

impl<MSG> Hash for StrongWidgetEntry<MSG> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.unique_id.hash(state)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct WidgetRef<MSG> {
    shared: StrongWidgetEntry<MSG>,
}

impl<MSG> WidgetRef<MSG> {
    pub fn unique_id(&self) -> u64 {
        self.shared.unique_id
    }
}

/// A set of widgets optimized for iteration.
pub(crate) struct WidgetSet<MSG> {
    unique_ids: FnvHashSet<u64>,
    entries: Vec<StrongWidgetEntry<MSG>>,
}

impl<MSG> WidgetSet<MSG> {
    pub fn new() -> Self {
        Self {
            unique_ids: FnvHashSet::default(),
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, widget_ref: &StrongWidgetEntry<MSG>) {
        if self.unique_ids.insert(widget_ref.unique_id) {
            self.entries.push(widget_ref.clone());
        }
    }

    pub fn remove(&mut self, widget_ref: &WidgetRef<MSG>) {
        if self.unique_ids.remove(&widget_ref.unique_id()) {
            let mut remove_i = None;
            for (i, entry) in self.entries.iter().enumerate() {
                if entry.unique_id == widget_ref.unique_id() {
                    remove_i = Some(i);
                    break;
                }
            }
            if let Some(i) = remove_i {
                self.entries.remove(i);
            }
        }
    }

    pub fn iter_mut(&mut self) -> impl IntoIterator<Item = RefMut<'_, Box<dyn Widget<MSG>>>> {
        self.entries.iter_mut().map(|e| e.shared.borrow_mut())
    }
}

pub struct Canvas<MSG> {
    next_layer_id: u64,
    next_widget_id: u64,

    layers: FnvHashMap<LayerID, StrongLayerEntry<MSG>>,
    layers_ordered: Vec<(i32, Vec<StrongLayerEntry<MSG>>)>,

    dirty_layers: FnvHashSet<LayerID>,

    widgets: FnvHashSet<StrongWidgetEntry<MSG>>,
    last_widget_that_captured_mouse: Option<(StrongWidgetEntry<MSG>, Option<LockMousePointerType>)>,
    widget_with_text_comp_listen: Option<StrongWidgetEntry<MSG>>,
    widgets_with_keyboard_listen: WidgetSet<MSG>,
    widgets_scheduled_for_animation: WidgetSet<MSG>,

    msg_out_queue: Vec<MSG>,

    do_repack_layers: bool,
}

impl<MSG> Canvas<MSG> {
    pub fn new() -> Self {
        Self {
            next_layer_id: 0,
            next_widget_id: 0,
            layers: FnvHashMap::default(),
            layers_ordered: Vec::new(),
            dirty_layers: FnvHashSet::default(),
            widgets: FnvHashSet::default(),
            last_widget_that_captured_mouse: None,
            widget_with_text_comp_listen: None,
            widgets_with_keyboard_listen: WidgetSet::new(),
            widgets_scheduled_for_animation: WidgetSet::new(),
            msg_out_queue: Vec::new(),
            do_repack_layers: true,
        }
    }

    pub fn add_layer(
        &mut self,
        size: Size,
        z_order: i32,
        position: Point,
    ) -> Result<LayerID, LayerError> {
        let id = LayerID {
            id: self.next_layer_id,
            z_order,
        };

        let layer = StrongLayerEntry {
            shared: Rc::new(RefCell::new(Layer::new(id, size, position)?)),
        };
        self.layers.insert(id, layer.clone());

        self.next_layer_id += 1;

        let mut existing_z_order_i = None;
        let mut insert_i = 0;
        for (i, (z_order, _)) in self.layers_ordered.iter().enumerate() {
            if id.z_order == *z_order {
                existing_z_order_i = Some(i);
                break;
            } else if id.z_order > *z_order {
                insert_i = i + 1;
            }
        }
        if let Some(i) = existing_z_order_i {
            self.layers_ordered[i].1.push(layer);
        } else {
            self.layers_ordered
                .insert(insert_i, (id.z_order, vec![layer]));
        }

        self.do_repack_layers = true;

        Ok(id)
    }

    pub fn remove_layer(&mut self, id: LayerID) -> Result<(), ()> {
        if let Some(layer) = self.layers.get(&id) {
            if !layer.borrow().is_empty() {
                // TODO: Custom error
                return Err(());
            }
        } else {
            // TODO: Custom error
            return Err(());
        }

        let mut remove_z_order_i = None;
        for (z_order_i, (z_order, layers)) in self.layers_ordered.iter_mut().enumerate() {
            if id.z_order == *z_order {
                let mut remove_i = None;
                for (i, layer) in layers.iter().enumerate() {
                    if layer.borrow().id == id {
                        remove_i = Some(i);
                        break;
                    }
                }
                if let Some(i) = remove_i {
                    layers.remove(i);

                    if layers.is_empty() {
                        remove_z_order_i = Some(z_order_i);
                    }
                }

                break;
            }
        }
        if let Some(i) = remove_z_order_i {
            self.layers_ordered.remove(i);
        }

        self.dirty_layers.remove(&id);

        self.do_repack_layers = true;

        Ok(())
    }

    pub fn set_layer_position(
        &mut self,
        layer: LayerID,
        position: Point,
    ) -> Result<(), LayerError> {
        Ok(self
            .layers
            .get_mut(&layer)
            .ok_or_else(|| LayerError::LayerWithIDNotFound(layer))?
            .borrow_mut()
            .set_position(position))
    }

    pub fn set_layer_size(&mut self, layer: LayerID, size: Size) -> Result<(), LayerError> {
        self.layers
            .get_mut(&layer)
            .ok_or_else(|| LayerError::LayerWithIDNotFound(layer))?
            .borrow_mut()
            .set_size(size, &mut self.dirty_layers)
    }

    pub fn set_layer_visible(&mut self, layer: LayerID, visible: bool) -> Result<(), LayerError> {
        Ok(self
            .layers
            .get_mut(&layer)
            .ok_or_else(|| LayerError::LayerWithIDNotFound(layer))?
            .borrow_mut()
            .set_visible(visible, &mut self.dirty_layers))
    }

    pub fn add_container_region(
        &mut self,
        layer: LayerID,
        size: Size,
        internal_anchor: Anchor,
        parent_anchor: Anchor,
        parent_anchor_type: ParentAnchorType,
        anchor_offset: Point,
        visible: bool,
    ) -> Result<ContainerRegionID, ()> {
        self.layers
            .get_mut(&layer)
            .ok_or_else(|| ())?
            .borrow_mut()
            .add_container_region(
                size,
                internal_anchor,
                parent_anchor,
                parent_anchor_type,
                anchor_offset,
                visible,
            )
    }

    pub fn remove_container_region(
        &mut self,
        layer: LayerID,
        region: ContainerRegionID,
    ) -> Result<(), ()> {
        self.layers
            .get_mut(&layer)
            .ok_or_else(|| ())?
            .borrow_mut()
            .remove_container_region(region, &mut self.dirty_layers)
    }

    pub fn modify_container_region(
        &mut self,
        layer: LayerID,
        region: ContainerRegionID,
        new_size: Option<Size>,
        new_internal_anchor: Option<Anchor>,
        new_parent_anchor: Option<Anchor>,
        new_anchor_offset: Option<Point>,
    ) -> Result<(), ()> {
        self.layers
            .get_mut(&layer)
            .ok_or_else(|| ())?
            .borrow_mut()
            .modify_container_region(
                region,
                new_size,
                new_internal_anchor,
                new_parent_anchor,
                new_anchor_offset,
                &mut self.dirty_layers,
            )
    }

    pub fn set_container_region_visibility(
        &mut self,
        layer: LayerID,
        region: ContainerRegionID,
        visible: bool,
    ) -> Result<(), ()> {
        self.layers
            .get_mut(&layer)
            .ok_or_else(|| ())?
            .borrow_mut()
            .set_container_region_visibility(region, visible, &mut self.dirty_layers)
    }

    pub fn mark_container_region_dirty(
        &mut self,
        layer: LayerID,
        region: ContainerRegionID,
    ) -> Result<(), ()> {
        self.layers
            .get_mut(&layer)
            .ok_or_else(|| ())?
            .borrow_mut()
            .mark_container_region_dirty(region, &mut self.dirty_layers)
    }

    pub fn add_widget(
        &mut self,
        mut widget: Box<dyn Widget<MSG>>,
        layer: LayerID,
        region: RegionInfo,
    ) -> Result<WidgetRef<MSG>, ()> {
        let info = widget.on_added(&mut self.msg_out_queue);

        let id = self.next_widget_id;
        self.next_widget_id += 1;

        let mut layer_entry = if let Some(layer) = self.layers.get(&layer) {
            layer.clone()
        } else {
            // TODO: custom error
            return Err(());
        };

        let widget_entry = StrongWidgetEntry {
            shared: Rc::new(RefCell::new(widget)),
            assigned_layer: layer_entry.downgrade(),
            unique_id: id,
        };

        layer_entry.borrow_mut().add_widget_region(
            widget_entry.clone(),
            region,
            info.listen_to_mouse_events,
            info.region_type,
            info.visible,
            &mut self.dirty_layers,
        )?;

        self.widgets.insert(widget_entry.clone());

        let set_text_comp = match info.keyboard_events_listen {
            KeyboardEventsListen::None => false,
            KeyboardEventsListen::Keys => {
                self.widgets_with_keyboard_listen.insert(&widget_entry);
                false
            }
            KeyboardEventsListen::TextComposition => true,
            KeyboardEventsListen::KeysAndTextComposition => {
                self.widgets_with_keyboard_listen.insert(&widget_entry);
                true
            }
        };

        if set_text_comp {
            if let Some(last_widget) = self.widget_with_text_comp_listen.take() {
                // TODO: send composition end event to widget
            }

            self.widget_with_text_comp_listen = Some(widget_entry.clone());
            // TODO: send composition start event to widget
        }

        if info.recieve_next_animation_event {
            self.widgets_scheduled_for_animation.insert(&widget_entry);
        }

        Ok(WidgetRef {
            shared: widget_entry,
        })
    }

    pub fn modify_widget_region(
        &mut self,
        widget_ref: &mut WidgetRef<MSG>,
        new_size: Option<Size>,
        new_internal_anchor: Option<Anchor>,
        new_parent_anchor: Option<Anchor>,
        new_anchor_offset: Option<Point>,
    ) {
        widget_ref
            .shared
            .assigned_layer
            .upgrade()
            .unwrap()
            .borrow_mut()
            .modify_widget_region(
                &widget_ref.shared,
                new_size,
                new_internal_anchor,
                new_parent_anchor,
                new_anchor_offset,
                &mut self.dirty_layers,
            )
            .unwrap();
    }

    pub fn set_widget_visibility(&mut self, widget_ref: &mut WidgetRef<MSG>, visible: bool) {
        widget_ref
            .shared
            .assigned_layer
            .upgrade()
            .unwrap()
            .borrow_mut()
            .set_widget_region_visibility(widget_ref, visible, &mut self.dirty_layers)
            .unwrap();
    }

    pub fn remove_widget(&mut self, mut widget_ref: WidgetRef<MSG>) {
        // Remove this widget from its assigned layer.
        widget_ref
            .shared
            .assigned_layer
            .upgrade()
            .unwrap()
            .borrow_mut()
            .remove_widget_region(&widget_ref.shared, &mut self.dirty_layers);

        // Remove this widget from all active event listeners.
        if let Some(w) = self.last_widget_that_captured_mouse.take() {
            if w.0.unique_id != widget_ref.unique_id() {
                self.last_widget_that_captured_mouse = Some(w);
            }
        }
        if let Some(w) = self.widget_with_text_comp_listen.take() {
            if w.unique_id != widget_ref.unique_id() {
                self.widget_with_text_comp_listen = Some(w);
            }
        }
        self.widgets_with_keyboard_listen.remove(&widget_ref);
        self.widgets_scheduled_for_animation.remove(&widget_ref);

        widget_ref
            .shared
            .borrow_mut()
            .on_removed(&mut self.msg_out_queue);
    }

    fn pack_layers(&mut self) {

        // TODO
    }
}
