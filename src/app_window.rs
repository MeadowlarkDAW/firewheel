use fnv::{FnvHashMap, FnvHashSet};
use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::ffi::c_void;
use std::hash::Hash;
use std::rc::{Rc, Weak};

use crate::anchor::Anchor;
use crate::event::{InputEvent, KeyboardEventsListen};
use crate::layer::{Layer, LayerError, LayerID, WeakRegionTreeEntry};
use crate::renderer::{LayerRenderer, Renderer};
use crate::size::PhysicalSize;
use crate::widget::{SetPointerLockType, Widget};
use crate::{
    ContainerRegionRef, EventCapturedStatus, Point, RegionInfo, ScaleFactor, Size, WidgetRequests,
};

pub(crate) struct StrongLayerEntry<MSG> {
    shared: Rc<RefCell<Layer<MSG>>>,
}

impl<MSG> StrongLayerEntry<MSG> {
    fn borrow(&self) -> Ref<'_, Layer<MSG>> {
        RefCell::borrow(&self.shared)
    }

    pub fn borrow_mut(&mut self) -> RefMut<'_, Layer<MSG>> {
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
    pub fn new() -> Self {
        Self {
            shared: Weak::new(),
        }
    }

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
    assigned_region: WeakRegionTreeEntry<MSG>,
    unique_id: u64,
}

impl<MSG> StrongWidgetEntry<MSG> {
    // Used by the unit tests.
    #[allow(unused)]
    pub fn new(widget: Box<dyn Widget<MSG>>, unique_id: u64) -> Self {
        Self {
            shared: Rc::new(RefCell::new(widget)),
            assigned_layer: WeakLayerEntry {
                shared: Weak::new(),
            },
            assigned_region: WeakRegionTreeEntry::new(),
            unique_id,
        }
    }

    pub fn borrow_mut(&mut self) -> RefMut<'_, Box<dyn Widget<MSG>>> {
        RefCell::borrow_mut(&self.shared)
    }

    pub fn unique_id(&self) -> u64 {
        self.unique_id
    }

    pub fn set_assigned_region(&mut self, region: WeakRegionTreeEntry<MSG>) {
        self.assigned_region = region;
    }

    pub fn assigned_region(&self) -> &WeakRegionTreeEntry<MSG> {
        &self.assigned_region
    }

    pub fn assigned_region_mut(&mut self) -> &mut WeakRegionTreeEntry<MSG> {
        &mut self.assigned_region
    }
}

impl<MSG> Clone for StrongWidgetEntry<MSG> {
    fn clone(&self) -> Self {
        Self {
            shared: Rc::clone(&self.shared),
            assigned_layer: self.assigned_layer.clone(),
            assigned_region: self.assigned_region.clone(),
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

    pub fn insert(&mut self, widget_entry: &StrongWidgetEntry<MSG>) {
        if self.unique_ids.insert(widget_entry.unique_id) {
            self.entries.push(widget_entry.clone());
        }
    }

    pub fn remove(&mut self, widget_entry: &StrongWidgetEntry<MSG>) {
        if self.unique_ids.remove(&widget_entry.unique_id()) {
            let mut remove_i = None;
            for (i, entry) in self.entries.iter().enumerate() {
                if entry.unique_id == widget_entry.unique_id() {
                    remove_i = Some(i);
                    break;
                }
            }
            if let Some(i) = remove_i {
                self.entries.remove(i);
            }
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut StrongWidgetEntry<MSG>> {
        self.entries.iter_mut()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.unique_ids.clear();
        self.entries.clear();
    }

    /// Used for testing purposes
    #[allow(unused)]
    pub fn contains(&self, widget_entry: &StrongWidgetEntry<MSG>) -> bool {
        self.unique_ids.contains(&widget_entry.unique_id)
    }
}

pub struct AppWindow<MSG> {
    pub(crate) layers_ordered: Vec<(i32, Vec<StrongLayerEntry<MSG>>)>,
    pub(crate) layer_renderers_to_clean_up: Vec<LayerRenderer>,

    next_layer_id: u64,
    next_widget_id: u64,

    layers: FnvHashMap<LayerID, StrongLayerEntry<MSG>>,

    widgets: FnvHashSet<StrongWidgetEntry<MSG>>,
    widget_with_pointer_lock: Option<(StrongWidgetEntry<MSG>, SetPointerLockType)>,
    widgets_to_send_input_event: Vec<(StrongWidgetEntry<MSG>, InputEvent)>,
    widget_with_text_comp_listen: Option<StrongWidgetEntry<MSG>>,
    widgets_with_keyboard_listen: WidgetSet<MSG>,
    widgets_scheduled_for_animation: WidgetSet<MSG>,
    widgets_with_pointer_down_listen: WidgetSet<MSG>,
    widgets_to_remove_from_animation: Vec<StrongWidgetEntry<MSG>>,
    widget_requests: Vec<(StrongWidgetEntry<MSG>, WidgetRequests)>,
    widgets_just_shown: WidgetSet<MSG>,
    widgets_just_hidden: WidgetSet<MSG>,

    renderer: Option<Renderer>,
    scale_factor: ScaleFactor,
    window_visibility: bool,

    do_repack_layers: bool,
}

impl<MSG> AppWindow<MSG> {
    pub unsafe fn new_from_function<F>(scale_factor: ScaleFactor, load_fn: F) -> Self
    where
        F: FnMut(&str) -> *const c_void,
    {
        let renderer = Renderer::new_from_function(load_fn);

        Self {
            next_layer_id: 0,
            next_widget_id: 0,
            layers: FnvHashMap::default(),
            layers_ordered: Vec::new(),
            widgets: FnvHashSet::default(),
            widget_with_pointer_lock: None,
            widgets_to_send_input_event: Vec::new(),
            widget_with_text_comp_listen: None,
            widgets_with_keyboard_listen: WidgetSet::new(),
            widgets_scheduled_for_animation: WidgetSet::new(),
            widgets_with_pointer_down_listen: WidgetSet::new(),
            widgets_to_remove_from_animation: Vec::new(),
            widget_requests: Vec::new(),
            widgets_just_shown: WidgetSet::new(),
            widgets_just_hidden: WidgetSet::new(),
            layer_renderers_to_clean_up: Vec::new(),
            renderer: Some(renderer),
            scale_factor,
            window_visibility: true,
            do_repack_layers: true,
        }
    }

    pub fn add_layer(
        &mut self,
        size: Size,
        z_order: i32,
        outer_position: Point,
        inner_position: Point,
        explicit_visibility: bool,
    ) -> Result<LayerID, LayerError> {
        let id = LayerID {
            id: self.next_layer_id,
            z_order,
        };

        let layer = StrongLayerEntry {
            shared: Rc::new(RefCell::new(Layer::new(
                id,
                size,
                outer_position,
                inner_position,
                explicit_visibility,
                self.window_visibility,
                self.scale_factor,
            )?)),
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

    // TODO: Custom error type
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
                    let mut entry = layers.remove(i);

                    if let Some(renderer) = entry.borrow_mut().renderer.take() {
                        self.layer_renderers_to_clean_up.push(renderer);
                    }

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

        self.do_repack_layers = true;

        Ok(())
    }

    pub fn set_layer_outer_position(
        &mut self,
        layer: LayerID,
        position: Point,
    ) -> Result<(), LayerError> {
        Ok(self
            .layers
            .get_mut(&layer)
            .ok_or_else(|| LayerError::LayerWithIDNotFound(layer))?
            .borrow_mut()
            .set_outer_position(position, self.scale_factor))
    }

    pub fn set_layer_inner_position(
        &mut self,
        layer: LayerID,
        position: Point,
        msg_out_queue: &mut Vec<MSG>,
    ) -> Result<(), LayerError> {
        {
            self.layers
                .get_mut(&layer)
                .ok_or_else(|| LayerError::LayerWithIDNotFound(layer))?
                .borrow_mut()
                .set_inner_position(
                    position,
                    &mut self.widgets_just_shown,
                    &mut self.widgets_just_hidden,
                );
        }

        self.handle_visibility_changes(msg_out_queue);

        Ok(())
    }

    pub fn set_layer_size(
        &mut self,
        layer: LayerID,
        size: Size,
        msg_out_queue: &mut Vec<MSG>,
    ) -> Result<(), LayerError> {
        self.layers
            .get_mut(&layer)
            .ok_or_else(|| LayerError::LayerWithIDNotFound(layer))?
            .borrow_mut()
            .set_size(
                size,
                self.scale_factor,
                &mut self.widgets_just_shown,
                &mut self.widgets_just_hidden,
            );

        self.handle_visibility_changes(msg_out_queue);

        Ok(())
    }

    pub fn set_layer_explicit_visibility(
        &mut self,
        layer: LayerID,
        explicit_visibility: bool,
        msg_out_queue: &mut Vec<MSG>,
    ) -> Result<(), LayerError> {
        self.layers
            .get_mut(&layer)
            .ok_or_else(|| LayerError::LayerWithIDNotFound(layer))?
            .borrow_mut()
            .set_explicit_visibility(
                explicit_visibility,
                &mut self.widgets_just_shown,
                &mut self.widgets_just_hidden,
            );

        self.handle_visibility_changes(msg_out_queue);

        Ok(())
    }

    pub fn set_window_visibility(&mut self, visible: bool, msg_out_queue: &mut Vec<MSG>) {
        if self.window_visibility != visible {
            self.window_visibility = visible;

            for (_z_order, layers) in self.layers_ordered.iter_mut() {
                for layer_entry in layers.iter_mut() {
                    layer_entry.borrow_mut().set_window_visibility(
                        visible,
                        &mut self.widgets_just_shown,
                        &mut self.widgets_just_hidden,
                    );
                }
            }

            self.handle_visibility_changes(msg_out_queue);
        }
    }

    // TODO: Custom error type
    pub fn add_container_region(
        &mut self,
        layer: LayerID,
        region_info: RegionInfo<MSG>,
        explicit_visibility: bool,
    ) -> Result<ContainerRegionRef<MSG>, ()> {
        let layer_entry = self.layers.get_mut(&layer).ok_or_else(|| ())?;
        let weak_layer_entry = layer_entry.downgrade();

        layer_entry
            .borrow_mut()
            .add_container_region(
                region_info,
                explicit_visibility,
                // No widgets will ever be shown or hidden as a result of
                // adding a container region.
                &mut self.widgets_just_shown,
                &mut self.widgets_just_hidden,
            )
            .map(|mut container_ref| {
                container_ref.assigned_layer = weak_layer_entry;
                container_ref
            })
    }

    // TODO: Custom error type
    pub fn remove_container_region(&mut self, region: ContainerRegionRef<MSG>) -> Result<(), ()> {
        region
            .assigned_layer
            .upgrade()
            .ok_or_else(|| ())?
            .borrow_mut()
            .remove_container_region(region)
    }

    // TODO: Custom error type
    pub fn modify_container_region(
        &mut self,
        region: &mut ContainerRegionRef<MSG>,
        new_size: Option<Size>,
        new_internal_anchor: Option<Anchor>,
        new_parent_anchor: Option<Anchor>,
        new_anchor_offset: Option<Point>,
        msg_out_queue: &mut Vec<MSG>,
    ) -> Result<(), ()> {
        region
            .assigned_layer
            .upgrade()
            .ok_or_else(|| ())?
            .borrow_mut()
            .modify_container_region(
                region,
                new_size,
                new_internal_anchor,
                new_parent_anchor,
                new_anchor_offset,
                &mut self.widgets_just_shown,
                &mut self.widgets_just_hidden,
            )?;

        self.handle_visibility_changes(msg_out_queue);

        Ok(())
    }

    // TODO: Custom error type
    pub fn set_container_region_explicit_visibility(
        &mut self,
        region: &mut ContainerRegionRef<MSG>,
        visible: bool,
        msg_out_queue: &mut Vec<MSG>,
    ) -> Result<(), ()> {
        region
            .assigned_layer
            .upgrade()
            .ok_or_else(|| ())?
            .borrow_mut()
            .set_container_region_explicit_visibility(
                region,
                visible,
                &mut self.widgets_just_shown,
                &mut self.widgets_just_hidden,
            )?;

        self.handle_visibility_changes(msg_out_queue);

        Ok(())
    }

    // TODO: Custom error type
    pub fn mark_container_region_dirty(
        &mut self,
        region: &mut ContainerRegionRef<MSG>,
    ) -> Result<(), ()> {
        region
            .assigned_layer
            .upgrade()
            .ok_or_else(|| ())?
            .borrow_mut()
            .mark_container_region_dirty(region)
    }

    // TODO: Custom error type
    pub fn add_widget(
        &mut self,
        mut widget: Box<dyn Widget<MSG>>,
        layer: LayerID,
        region_info: RegionInfo<MSG>,
        explicit_visibility: bool,
        msg_out_queue: &mut Vec<MSG>,
    ) -> Result<WidgetRef<MSG>, ()> {
        let info = widget.on_added(msg_out_queue);

        let id = self.next_widget_id;
        self.next_widget_id += 1;

        let mut layer_entry = if let Some(layer) = self.layers.get(&layer) {
            layer.clone()
        } else {
            // TODO: custom error
            return Err(());
        };

        let mut widget_entry = StrongWidgetEntry {
            shared: Rc::new(RefCell::new(widget)),
            assigned_layer: layer_entry.downgrade(),
            assigned_region: WeakRegionTreeEntry::new(),
            unique_id: id,
        };

        layer_entry.borrow_mut().add_widget_region(
            &mut widget_entry,
            region_info,
            info.region_type,
            explicit_visibility,
            &mut self.widgets_just_shown,
            &mut self.widgets_just_hidden,
        )?;

        self.widgets.insert(widget_entry.clone());

        self.handle_visibility_changes(msg_out_queue);

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
        msg_out_queue: &mut Vec<MSG>,
    ) {
        widget_ref
            .shared
            .assigned_layer
            .upgrade()
            .unwrap()
            .borrow_mut()
            .modify_widget_region(
                &mut widget_ref.shared,
                new_size,
                new_internal_anchor,
                new_parent_anchor,
                new_anchor_offset,
                &mut self.widgets_just_shown,
                &mut self.widgets_just_hidden,
            )
            .unwrap();

        self.handle_visibility_changes(msg_out_queue);
    }

    pub fn set_widget_explicit_visibility(
        &mut self,
        widget_ref: &mut WidgetRef<MSG>,
        visible: bool,
        msg_out_queue: &mut Vec<MSG>,
    ) {
        widget_ref
            .shared
            .assigned_layer
            .upgrade()
            .unwrap()
            .borrow_mut()
            .set_widget_explicit_visibility(
                &mut widget_ref.shared,
                visible,
                &mut self.widgets_just_shown,
                &mut self.widgets_just_hidden,
            )
            .unwrap();

        self.handle_visibility_changes(msg_out_queue);
    }

    pub fn remove_widget(&mut self, mut widget_ref: WidgetRef<MSG>, msg_out_queue: &mut Vec<MSG>) {
        // Remove this widget from its assigned layer.
        widget_ref
            .shared
            .assigned_layer
            .upgrade()
            .unwrap()
            .borrow_mut()
            .remove_widget_region(
                &mut widget_ref.shared,
                &mut self.widgets_just_shown,
                &mut self.widgets_just_hidden,
            );

        // Remove this widget from all active event listeners.
        self.widgets_scheduled_for_animation
            .remove(&widget_ref.shared);
        self.widgets_with_keyboard_listen.remove(&widget_ref.shared);
        self.widgets_with_pointer_down_listen
            .remove(&widget_ref.shared);
        if let Some(w) = self.widget_with_pointer_lock.take() {
            if w.0.unique_id != widget_ref.unique_id() {
                self.widget_with_pointer_lock = Some(w);
            }
        }
        if let Some(w) = self.widget_with_text_comp_listen.take() {
            if w.unique_id != widget_ref.unique_id() {
                self.widget_with_text_comp_listen = Some(w);
            }
        }

        widget_ref.shared.borrow_mut().on_removed(msg_out_queue);
    }

    pub fn send_user_event_to_widget(
        &mut self,
        widget_ref: &mut WidgetRef<MSG>,
        event: Box<dyn Any>,
        msg_out_queue: &mut Vec<MSG>,
    ) {
        let res = {
            widget_ref
                .shared
                .borrow_mut()
                .on_user_event(event, msg_out_queue)
        };
        if let Some(requests) = res {
            self.handle_widget_requests(&widget_ref.shared, requests);
        }
    }

    pub fn mark_widget_dirty(&mut self, widget_ref: &mut WidgetRef<MSG>) {
        widget_ref
            .shared
            .assigned_layer
            .upgrade()
            .unwrap()
            .borrow_mut()
            .mark_widget_region_dirty(&widget_ref.shared)
            .unwrap();
    }

    pub fn set_scale_factor(&mut self, scale_factor: ScaleFactor) {
        // TODO
    }

    pub fn handle_input_event(
        &mut self,
        event: &InputEvent,
        msg_out_queue: &mut Vec<MSG>,
    ) -> InputEventResult {
        match event {
            InputEvent::Animation(_) => {
                let mut widgets_to_remove_from_animation: Vec<StrongWidgetEntry<MSG>> = Vec::new();
                let mut widget_requests: Vec<(StrongWidgetEntry<MSG>, WidgetRequests)> = Vec::new();
                std::mem::swap(
                    &mut widgets_to_remove_from_animation,
                    &mut self.widgets_to_remove_from_animation,
                );
                std::mem::swap(&mut widget_requests, &mut self.widget_requests);

                for widget_entry in self.widgets_scheduled_for_animation.iter_mut() {
                    let res = {
                        widget_entry
                            .borrow_mut()
                            .on_input_event(event, msg_out_queue)
                    };
                    if let EventCapturedStatus::Captured(requests) = res {
                        widget_requests.push((widget_entry.clone(), requests));
                    } else {
                        widgets_to_remove_from_animation.push(widget_entry.clone());
                    }
                }

                for (widget_entry, requests) in widget_requests.drain(..) {
                    self.handle_widget_requests(&widget_entry, requests);
                }
                for widget_entry in widgets_to_remove_from_animation.drain(..) {
                    self.widgets_scheduled_for_animation.remove(&widget_entry);
                }

                std::mem::swap(
                    &mut widgets_to_remove_from_animation,
                    &mut self.widgets_to_remove_from_animation,
                );
                std::mem::swap(&mut widget_requests, &mut self.widget_requests);
            }
            InputEvent::Pointer(mut e) => {
                let pointer_locked_in_place = self
                    .widget_with_pointer_lock
                    .as_ref()
                    .map(|(_, lock_type)| {
                        *lock_type == SetPointerLockType::LockInPlaceAndHideCursor
                    })
                    .unwrap_or(false);

                if pointer_locked_in_place {
                    // Remove the position data when the pointer is locked in place.
                    e.position = Point::default();

                    let mut widget_entry =
                        self.widget_with_pointer_lock.as_ref().unwrap().0.clone();
                    let res = {
                        widget_entry
                            .borrow_mut()
                            .on_input_event(event, msg_out_queue)
                    };
                    if let EventCapturedStatus::Captured(requests) = res {
                        self.handle_widget_requests(&widget_entry, requests);
                    }
                } else {
                    if !self.widgets_with_pointer_down_listen.is_empty() {
                        if e.any_button_just_pressed() {
                            let mut widget_requests: Vec<(StrongWidgetEntry<MSG>, WidgetRequests)> =
                                Vec::new();
                            std::mem::swap(&mut widget_requests, &mut self.widget_requests);

                            for widget_entry in self.widgets_with_pointer_down_listen.iter_mut() {
                                let res = {
                                    widget_entry
                                        .borrow_mut()
                                        .on_input_event(event, msg_out_queue)
                                };
                                if let EventCapturedStatus::Captured(requests) = res {
                                    widget_requests.push((widget_entry.clone(), requests));
                                }
                            }

                            for (widget_entry, requests) in widget_requests.drain(..) {
                                self.handle_widget_requests(&widget_entry, requests);
                            }

                            std::mem::swap(&mut widget_requests, &mut self.widget_requests);
                        }
                    }

                    let mut widget_requests = None;
                    for (_z_index, layers) in self.layers_ordered.iter_mut().rev() {
                        for layer in layers.iter_mut() {
                            if let Some(captured_res) =
                                layer.borrow_mut().handle_pointer_event(e, msg_out_queue)
                            {
                                widget_requests = Some(captured_res);
                                break;
                            }
                        }
                        if widget_requests.is_some() {
                            break;
                        }
                    }

                    if let Some((widget_entry, requests)) = widget_requests {
                        self.handle_widget_requests(&widget_entry, requests);
                    }
                }
            }
            InputEvent::PointerUnlocked => {
                let mut requests = None;
                if let Some((mut last_widget, _lock_type)) = self.widget_with_pointer_lock.take() {
                    let res = {
                        last_widget
                            .borrow_mut()
                            .on_input_event(event, msg_out_queue)
                    };
                    if let EventCapturedStatus::Captured(r) = res {
                        requests = Some((last_widget.clone(), r));
                    }
                }

                if let Some((widget_entry, requests)) = requests {
                    self.handle_widget_requests(&widget_entry, requests);
                }
            }
            InputEvent::Keyboard(_) => {
                let mut widget_requests: Vec<(StrongWidgetEntry<MSG>, WidgetRequests)> = Vec::new();
                std::mem::swap(&mut widget_requests, &mut self.widget_requests);

                for widget_entry in self.widgets_with_keyboard_listen.iter_mut() {
                    let res = {
                        widget_entry
                            .borrow_mut()
                            .on_input_event(event, msg_out_queue)
                    };
                    if let EventCapturedStatus::Captured(requests) = res {
                        widget_requests.push((widget_entry.clone(), requests));
                    }
                }

                for (widget_entry, requests) in widget_requests.drain(..) {
                    self.handle_widget_requests(&widget_entry, requests);
                }

                std::mem::swap(&mut widget_requests, &mut self.widget_requests);
            }
            InputEvent::TextComposition(_) => {
                let mut requests = None;
                if let Some(widget_entry) = &mut self.widget_with_text_comp_listen {
                    let res = {
                        widget_entry
                            .borrow_mut()
                            .on_input_event(event, msg_out_queue)
                    };
                    if let EventCapturedStatus::Captured(r) = res {
                        requests = Some((widget_entry.clone(), r));
                    }
                }

                if let Some((widget_entry, requests)) = requests {
                    self.handle_widget_requests(&widget_entry, requests);
                }
            }
            e => {
                log::warn!("Input event {:?} is reserved for internal use", e);
            }
        }

        // Handle any extra events that have occurred as a result of handling
        // widget requests.
        let mut widgets_to_send_input_event: Vec<(StrongWidgetEntry<MSG>, InputEvent)> = Vec::new();
        std::mem::swap(
            &mut widgets_to_send_input_event,
            &mut self.widgets_to_send_input_event,
        );
        for (mut widget_entry, event) in widgets_to_send_input_event.drain(..) {
            let res = {
                widget_entry
                    .borrow_mut()
                    .on_input_event(&event, msg_out_queue)
            };
            if let EventCapturedStatus::Captured(requests) = res {
                self.handle_widget_requests(&widget_entry, requests);
            }
        }
        widgets_to_send_input_event.append(&mut self.widgets_to_send_input_event);
        std::mem::swap(
            &mut widgets_to_send_input_event,
            &mut self.widgets_to_send_input_event,
        );

        let lock_pointer_in_place = self
            .widget_with_pointer_lock
            .as_ref()
            .map(|(_, lock_type)| *lock_type == SetPointerLockType::LockInPlaceAndHideCursor)
            .unwrap_or(false);

        InputEventResult {
            lock_pointer_in_place,
        }
    }

    pub fn render(&mut self, window_size: PhysicalSize) {
        let mut renderer = self.renderer.take().unwrap();

        renderer.render(self, window_size, self.scale_factor);

        self.renderer = Some(renderer);
    }

    fn handle_widget_requests(
        &mut self,
        widget_entry: &StrongWidgetEntry<MSG>,
        requests: WidgetRequests,
    ) {
        if requests.repaint {
            // Note, the widget won't actually get marked dirty if it is
            // currently hidden.
            widget_entry
                .assigned_layer
                .upgrade()
                .unwrap()
                .borrow_mut()
                .mark_widget_region_dirty(widget_entry)
                .unwrap();
        }
        if let Some(recieve_next_animation_event) = requests.set_recieve_next_animation_event {
            if recieve_next_animation_event {
                let is_visible = {
                    widget_entry
                        .assigned_region
                        .upgrade()
                        .unwrap()
                        .borrow()
                        .region
                        .is_visible()
                };
                if is_visible {
                    self.widgets_scheduled_for_animation.insert(widget_entry);
                }
            } else {
                self.widgets_scheduled_for_animation.remove(widget_entry);
            }
        }
        if let Some(listens) = requests.set_pointer_events_listen {
            widget_entry
                .assigned_layer
                .upgrade()
                .unwrap()
                .borrow_mut()
                .set_widget_region_listens_to_pointer_events(widget_entry, listens)
                .unwrap();
        }
        if let Some(set_keyboard_events_listen) = requests.set_keyboard_events_listen {
            let is_visible = {
                widget_entry
                    .assigned_region
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .region
                    .is_visible()
            };

            let set_text_comp = if is_visible {
                match set_keyboard_events_listen {
                    KeyboardEventsListen::None => {
                        self.widgets_with_keyboard_listen.remove(&widget_entry);
                        false
                    }
                    KeyboardEventsListen::Keys => {
                        self.widgets_with_keyboard_listen.insert(&widget_entry);
                        false
                    }
                    KeyboardEventsListen::TextComposition => {
                        self.widgets_with_keyboard_listen.remove(&widget_entry);
                        true
                    }
                    KeyboardEventsListen::KeysAndTextComposition => {
                        self.widgets_with_keyboard_listen.insert(&widget_entry);
                        true
                    }
                }
            } else {
                self.widgets_with_keyboard_listen.remove(&widget_entry);
                false
            };

            if set_text_comp {
                if let Some(last_widget) = self.widget_with_text_comp_listen.take() {
                    if last_widget.unique_id() != widget_entry.unique_id() {
                        self.widgets_to_send_input_event
                            .push((last_widget.clone(), InputEvent::TextCompositionUnfocused));
                        self.widgets_to_send_input_event
                            .push((widget_entry.clone(), InputEvent::TextCompositionFocused));

                        self.widget_with_text_comp_listen = Some(widget_entry.clone());
                    } else {
                        self.widget_with_text_comp_listen = Some(last_widget);
                    }
                } else {
                    self.widget_with_text_comp_listen = Some(widget_entry.clone());
                    self.widgets_to_send_input_event
                        .push((widget_entry.clone(), InputEvent::TextCompositionFocused));
                }
            } else {
                if let Some(last_widget) = self.widget_with_text_comp_listen.take() {
                    if last_widget.unique_id() == widget_entry.unique_id() {
                        self.widgets_to_send_input_event
                            .push((widget_entry.clone(), InputEvent::TextCompositionUnfocused));
                    } else {
                        self.widget_with_text_comp_listen = Some(last_widget);
                    }
                }
            }
        }
        if let Some(set_lock_type) = requests.set_pointer_lock {
            let is_visible = {
                widget_entry
                    .assigned_region
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .region
                    .is_visible()
            };

            if set_lock_type == SetPointerLockType::Unlock || !is_visible {
                if let Some((last_widget, lock_type)) = self.widget_with_pointer_lock.take() {
                    if last_widget.unique_id() == widget_entry.unique_id() {
                        self.widgets_to_send_input_event
                            .push((widget_entry.clone(), InputEvent::PointerUnlocked));
                    } else {
                        self.widget_with_pointer_lock = Some((last_widget, lock_type));
                    }
                }
            } else {
                if let Some((last_widget, _)) = &mut self.widget_with_pointer_lock {
                    if last_widget.unique_id() != widget_entry.unique_id() {
                        self.widgets_to_send_input_event
                            .push((last_widget.clone(), InputEvent::PointerUnlocked));
                    } else {
                        self.widget_with_pointer_lock = Some((widget_entry.clone(), set_lock_type));
                    }
                } else {
                    self.widget_with_pointer_lock = Some((widget_entry.clone(), set_lock_type));
                    self.widgets_to_send_input_event
                        .push((widget_entry.clone(), InputEvent::PointerLocked));
                }
            }
        }
        if let Some(set_pointer_down_listen) = requests.set_pointer_down_listen {
            let is_visible = {
                widget_entry
                    .assigned_region
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .region
                    .is_visible()
            };

            if set_pointer_down_listen && is_visible {
                self.widgets_with_pointer_down_listen.insert(&widget_entry);
            } else {
                self.widgets_with_pointer_down_listen.remove(&widget_entry);
            }
        }
    }

    fn handle_visibility_changes(&mut self, msg_out_queue: &mut Vec<MSG>) {
        // Handle widgets that have just been shown.
        let mut widget_requests: Vec<(StrongWidgetEntry<MSG>, WidgetRequests)> = Vec::new();
        std::mem::swap(&mut widget_requests, &mut self.widget_requests);
        for widget_entry in self.widgets_just_shown.iter_mut() {
            let status = {
                widget_entry
                    .borrow_mut()
                    .on_input_event(&InputEvent::VisibilityShown, msg_out_queue)
            };
            if let EventCapturedStatus::Captured(requests) = status {
                widget_requests.push((widget_entry.clone(), requests));
            }
        }
        self.widgets_just_shown.clear();
        for (widget_entry, requests) in widget_requests.drain(..) {
            self.handle_widget_requests(&widget_entry, requests);
        }
        std::mem::swap(&mut widget_requests, &mut self.widget_requests);

        // Handle widgets that have just been hidden.
        for widget_entry in self.widgets_just_hidden.iter_mut() {
            {
                widget_entry
                    .borrow_mut()
                    .on_visibility_hidden(msg_out_queue);
            }

            // Remove all event listeners for this widget (except for pointer
            // input events, because the region tree already culls pointer
            // input events from hidden widgets).
            self.widgets_scheduled_for_animation.remove(widget_entry);
            self.widgets_with_keyboard_listen.remove(widget_entry);
            self.widgets_with_pointer_down_listen.remove(widget_entry);
            if let Some((last_widget, lock_type)) = self.widget_with_pointer_lock.take() {
                if last_widget.unique_id() != widget_entry.unique_id() {
                    self.widget_with_pointer_lock = Some((last_widget, lock_type));
                }
            }
            if let Some(last_widget) = self.widget_with_text_comp_listen.take() {
                if last_widget.unique_id() != widget_entry.unique_id() {
                    self.widget_with_text_comp_listen = Some(last_widget);
                }
            }
        }
        self.widgets_just_hidden.clear();
    }
}

pub struct InputEventResult {
    pub lock_pointer_in_place: bool,
    // TODO: cursor icon
}
