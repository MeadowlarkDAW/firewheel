use fnv::FnvHashSet;

use crate::anchor::Anchor;
use crate::canvas::{StrongWidgetEntry, WidgetRef};
use crate::event::MouseEvent;
use crate::size::{Point, Size};
use crate::WidgetRequests;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::hash::Hash;

mod region_tree;
use region_tree::RegionTree;
pub use region_tree::{ContainerRegionID, ParentAnchorType};

/// The unique identifier for a layer.
#[derive(Debug, Clone, Copy)]
pub struct LayerID {
    /// The ID of this layer.
    pub(crate) id: u64,
    /// The z-order of this layer.
    pub z_order: i32,
}

impl LayerID {
    pub fn unique_id(&self) -> u64 {
        self.id
    }
}

impl PartialEq for LayerID {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for LayerID {}

impl PartialOrd for LayerID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LayerID {
    fn cmp(&self, other: &Self) -> Ordering {
        self.z_order.cmp(&other.z_order)
    }
}

impl Hash for LayerID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub(crate) struct Layer<MSG> {
    pub id: LayerID,

    region_tree: RegionTree<MSG>,
    position: Point,

    size: Size,

    visible: bool,
}

impl<MSG> Layer<MSG> {
    pub fn new(id: LayerID, size: Size, position: Point) -> Result<Self, LayerError> {
        Ok(Self {
            id,
            region_tree: RegionTree::new(size),
            position,
            size,
            visible: true,
        })
    }

    pub fn set_position(&mut self, position: Point) {
        self.position = position;
    }

    pub fn set_visible(&mut self, visible: bool, dirty_layers: &mut FnvHashSet<LayerID>) {
        if self.visible != visible {
            self.visible = visible;
            dirty_layers.insert(self.id);

            if visible {
                self.region_tree.layer_just_shown();
            } else {
                self.region_tree.layer_just_hidden();
            }
        }
    }

    pub fn set_size(
        &mut self,
        size: Size,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) -> Result<(), LayerError> {
        if self.size != size {
            self.size = size;
            self.region_tree.set_layer_size(size);
            dirty_layers.insert(self.id);
        }

        Ok(())
    }

    pub fn add_container_region(
        &mut self,
        size: Size,
        internal_anchor: Anchor,
        parent_anchor: Anchor,
        parent_anchor_type: ParentAnchorType,
        anchor_offset: Point,
        visible: bool,
    ) -> Result<ContainerRegionID, ()> {
        self.region_tree.new_container_region(
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
        id: ContainerRegionID,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) -> Result<(), ()> {
        self.region_tree.remove_container_region(id)?;

        if self.region_tree.is_dirty() {
            dirty_layers.insert(self.id);
        }

        Ok(())
    }

    pub fn modify_container_region(
        &mut self,
        id: ContainerRegionID,
        new_size: Option<Size>,
        new_internal_anchor: Option<Anchor>,
        new_parent_anchor: Option<Anchor>,
        new_anchor_offset: Option<Point>,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) -> Result<(), ()> {
        self.region_tree.modify_container_region(
            id,
            new_size,
            new_internal_anchor,
            new_parent_anchor,
            new_anchor_offset,
        )?;

        if self.region_tree.is_dirty() {
            dirty_layers.insert(self.id);
        }

        Ok(())
    }

    pub fn set_container_region_visibility(
        &mut self,
        id: ContainerRegionID,
        visible: bool,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) -> Result<(), ()> {
        self.region_tree
            .set_container_region_visibility(id, visible)?;

        if self.region_tree.is_dirty() {
            dirty_layers.insert(self.id);
        }

        Ok(())
    }

    pub fn mark_container_region_dirty(
        &mut self,
        id: ContainerRegionID,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) -> Result<(), ()> {
        self.region_tree.mark_container_region_dirty(id)?;

        if self.region_tree.is_dirty() {
            dirty_layers.insert(self.id);
        }

        Ok(())
    }

    pub fn insert_widget_region(
        &mut self,
        assigned_widget: StrongWidgetEntry<MSG>,
        size: Size,
        internal_anchor: Anchor,
        parent_anchor: Anchor,
        parent_anchor_type: ParentAnchorType,
        anchor_offset: Point,
        listens_to_mouse_events: bool,
        visible: bool,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) -> Result<(), ()> {
        self.region_tree.insert_widget_region(
            assigned_widget,
            size,
            internal_anchor,
            parent_anchor,
            parent_anchor_type,
            anchor_offset,
            listens_to_mouse_events,
            visible,
        )?;

        if self.region_tree.is_dirty() {
            dirty_layers.insert(self.id);
        }

        Ok(())
    }

    pub fn remove_widget_region(
        &mut self,
        widget: &WidgetRef<MSG>,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) -> Result<(), ()> {
        self.region_tree.remove_widget_region(widget)?;

        if self.region_tree.is_dirty() {
            dirty_layers.insert(self.id);
        }

        Ok(())
    }

    pub fn set_widget_region_visibility(
        &mut self,
        widget: &WidgetRef<MSG>,
        visible: bool,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) -> Result<(), ()> {
        self.region_tree
            .set_widget_region_visibility(widget, visible)?;

        if self.region_tree.is_dirty() {
            dirty_layers.insert(self.id);
        }

        Ok(())
    }

    pub fn mark_widget_region_dirty(
        &mut self,
        widget: &WidgetRef<MSG>,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) -> Result<(), ()> {
        self.region_tree.mark_widget_region_dirty(widget)?;

        if self.region_tree.is_dirty() {
            dirty_layers.insert(self.id);
        }

        Ok(())
    }

    pub fn handle_mouse_event(
        &mut self,
        mut event: MouseEvent,
    ) -> Option<(StrongWidgetEntry<MSG>, WidgetRequests<MSG>)> {
        if !self.visible {
            return None;
        }

        if event.position.x < self.position.x
            || event.position.y < self.position.y
            || event.position.x > self.position.x + self.size.width()
            || event.position.y > self.position.y + self.size.height()
        {
            return None;
        }

        // Remove this layer's offset from the position of the mouse event.
        event.position -= self.position;
        event.previous_position -= self.position;

        event.layer = self.id;

        self.region_tree.handle_mouse_event(event)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayerError {
    LayerWithIDNotFound(LayerID),
}

impl Error for LayerError {}

impl fmt::Display for LayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LayerWithIDNotFound(id) => {
                write!(f, "Could not find layer with ID {:?}", id)
            }
        }
    }
}
