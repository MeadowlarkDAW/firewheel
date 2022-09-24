use fnv::{FnvHashMap, FnvHashSet};
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::anchor::Anchor;
use crate::layer::{Layer, LayerError, LayerID};
use crate::widget::{Widget, WidgetID};
use crate::{ContainerRegionID, ParentAnchorType, Point, Size};

struct SharedLayerEntry<MSG> {
    shared: Rc<RefCell<Layer<MSG>>>,
}

impl<MSG> SharedLayerEntry<MSG> {
    fn borrow(&self) -> Ref<'_, Layer<MSG>> {
        RefCell::borrow(&self.shared)
    }

    fn borrow_mut(&mut self) -> RefMut<'_, Layer<MSG>> {
        RefCell::borrow_mut(&self.shared)
    }
}

impl<MSG> Clone for SharedLayerEntry<MSG> {
    fn clone(&self) -> Self {
        Self {
            shared: Rc::clone(&self.shared),
        }
    }
}

pub(crate) struct SharedWidgetEntry<MSG> {
    shared: Rc<RefCell<Box<dyn Widget<MSG>>>>,
}

impl<MSG> SharedWidgetEntry<MSG> {
    fn borrow(&self) -> Ref<'_, Box<dyn Widget<MSG>>> {
        RefCell::borrow(&self.shared)
    }

    pub(crate) fn borrow_mut(&mut self) -> RefMut<'_, Box<dyn Widget<MSG>>> {
        RefCell::borrow_mut(&self.shared)
    }
}

impl<MSG> Clone for SharedWidgetEntry<MSG> {
    fn clone(&self) -> Self {
        Self {
            shared: Rc::clone(&self.shared),
        }
    }
}

/// A set of widgets optimized for iteration.
pub(crate) struct WidgetSet<MSG> {
    set: FnvHashSet<WidgetID>,
    entries: Vec<(WidgetID, SharedWidgetEntry<MSG>)>,
}

impl<MSG> WidgetSet<MSG> {
    pub fn new() -> Self {
        Self {
            set: FnvHashSet::default(),
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, id: WidgetID, entry: SharedWidgetEntry<MSG>) {
        if self.set.insert(id) {
            self.entries.push((id, entry));
        }
    }

    pub fn remove(&mut self, id: WidgetID) {
        if self.set.remove(&id) {
            let mut remove_i = None;
            for (i, entry) in self.entries.iter().enumerate() {
                if entry.0 == id {
                    remove_i = Some(i);
                    break;
                }
            }
            if let Some(i) = remove_i {
                self.entries.remove(i);
            }
        }
    }

    pub fn iter_mut(&mut self) -> impl IntoIterator<Item = &'_ mut SharedWidgetEntry<MSG>> {
        self.entries.iter_mut().map(|(_, entry)| entry)
    }
}

pub struct Canvas<MSG> {
    next_layer_id: u64,

    layers: FnvHashMap<LayerID, SharedLayerEntry<MSG>>,
    layers_ordered: Vec<(i32, Vec<SharedLayerEntry<MSG>>)>,

    dirty_layers: FnvHashSet<LayerID>,

    widgets: FnvHashMap<WidgetID, SharedWidgetEntry<MSG>>,
    widget_with_text_comp_listen: Option<SharedWidgetEntry<MSG>>,
    widgets_with_keyboard_listen: WidgetSet<MSG>,
    widgets_scheduled_for_animation: WidgetSet<MSG>,

    do_repack_layers: bool,
}

impl<MSG> Canvas<MSG> {
    pub fn new() -> Self {
        Self {
            next_layer_id: 0,
            layers: FnvHashMap::default(),
            layers_ordered: Vec::new(),
            dirty_layers: FnvHashSet::default(),
            widgets: FnvHashMap::default(),
            widget_with_text_comp_listen: None,
            widgets_with_keyboard_listen: WidgetSet::new(),
            widgets_scheduled_for_animation: WidgetSet::new(),
            do_repack_layers: true,
        }
    }

    pub fn add_layer(
        &mut self,
        size: Size,
        min_size: Option<Size>,
        max_size: Option<Size>,
        z_order: i32,
        position: Point,
    ) -> Result<LayerID, LayerError> {
        let id = LayerID {
            id: self.next_layer_id,
            z_order,
        };

        let layer = SharedLayerEntry {
            shared: Rc::new(RefCell::new(Layer::new(
                id, size, min_size, max_size, position,
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

    pub fn remove_layer(&mut self, id: LayerID) {
        if self.layers.remove(&id).is_none() {
            return;
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

    pub fn modify_layer(
        &mut self,
        layer: LayerID,
        new_size: Option<Size>,
        new_min_size: Option<Size>,
        new_max_size: Option<Size>,
    ) -> Result<(), LayerError> {
        self.layers
            .get_mut(&layer)
            .ok_or_else(|| LayerError::LayerWithIDNotFound(layer))?
            .borrow_mut()
            .modify(new_size, new_min_size, new_max_size, &mut self.dirty_layers)
    }

    pub fn add_container_region(
        &mut self,
        layer: LayerID,
        size: Size,
        internal_anchor: Anchor,
        parent_anchor: Anchor,
        parent_anchor_type: ParentAnchorType,
        anchor_offset: Point,
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

    fn pack_layers(&mut self) {

        // TODO
    }
}
