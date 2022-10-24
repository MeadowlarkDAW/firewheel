use fnv::FnvHashSet;

use crate::anchor::Anchor;
use crate::canvas::StrongWidgetEntry;
use crate::event::PointerEvent;
use crate::glow_renderer::LayerRenderer;
use crate::size::{Point, Size};
use crate::{WidgetRegionType, WidgetRequests};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::hash::Hash;

mod region_tree;
use region_tree::RegionTree;
pub(crate) use region_tree::WeakRegionTreeEntry;
pub use region_tree::{ContainerRegionID, ParentAnchorType, RegionInfo};

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
    pub renderer: LayerRenderer,

    pub region_tree: RegionTree<MSG>,
    pub outer_position: Point,
}

impl<MSG> Layer<MSG> {
    pub fn new(
        id: LayerID,
        size: Size,
        outer_position: Point,
        inner_position: Point,
        explicit_visibility: bool,
    ) -> Result<Self, LayerError> {
        Ok(Self {
            id,
            renderer: LayerRenderer::new(),
            region_tree: RegionTree::new(size, inner_position, explicit_visibility),
            outer_position,
        })
    }

    pub fn set_outer_position(&mut self, position: Point) {
        self.outer_position = position;
    }

    pub fn set_inner_position(&mut self, position: Point, dirty_layers: &mut FnvHashSet<LayerID>) {
        self.region_tree.set_layer_inner_position(position);

        if self.region_tree.is_dirty() {
            dirty_layers.insert(self.id);
        }
    }

    pub fn set_explicit_visibility(
        &mut self,
        explicit_visibility: bool,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) {
        self.region_tree
            .set_layer_explicit_visibility(explicit_visibility);

        if self.region_tree.is_dirty() {
            dirty_layers.insert(self.id);
        }
    }

    pub fn set_size(&mut self, size: Size, dirty_layers: &mut FnvHashSet<LayerID>) {
        self.region_tree.set_layer_size(size);

        if self.region_tree.is_dirty() {
            dirty_layers.insert(self.id);
        }
    }

    pub fn add_container_region(
        &mut self,
        region_info: RegionInfo,
        explicit_visibility: bool,
    ) -> Result<ContainerRegionID, ()> {
        self.region_tree
            .add_container_region(region_info, explicit_visibility)
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

    pub fn set_container_region_explicit_visibility(
        &mut self,
        id: ContainerRegionID,
        visible: bool,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) -> Result<(), ()> {
        self.region_tree
            .set_container_region_explicit_visibility(id, visible)?;

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

    pub fn add_widget_region(
        &mut self,
        assigned_widget: &mut StrongWidgetEntry<MSG>,
        region_info: RegionInfo,
        region_type: WidgetRegionType,
        explicit_visibility: bool,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) -> Result<(), ()> {
        self.region_tree.add_widget_region(
            assigned_widget,
            region_info,
            region_type,
            explicit_visibility,
        )?;

        if self.region_tree.is_dirty() {
            dirty_layers.insert(self.id);
        }

        Ok(())
    }

    pub fn remove_widget_region(
        &mut self,
        widget: &StrongWidgetEntry<MSG>,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) {
        self.region_tree.remove_widget_region(widget);

        if self.region_tree.is_dirty() {
            dirty_layers.insert(self.id);
        }
    }

    pub fn modify_widget_region(
        &mut self,
        widget: &mut StrongWidgetEntry<MSG>,
        new_size: Option<Size>,
        new_internal_anchor: Option<Anchor>,
        new_parent_anchor: Option<Anchor>,
        new_anchor_offset: Option<Point>,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) -> Result<(), ()> {
        self.region_tree.modify_widget_region(
            widget,
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

    pub fn set_widget_explicit_visibility(
        &mut self,
        widget: &mut StrongWidgetEntry<MSG>,
        visible: bool,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) -> Result<(), ()> {
        self.region_tree
            .set_widget_explicit_visibility(widget, visible)?;

        if self.region_tree.is_dirty() {
            dirty_layers.insert(self.id);
        }

        Ok(())
    }

    pub fn mark_widget_region_dirty(
        &mut self,
        widget: &StrongWidgetEntry<MSG>,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) -> Result<(), ()> {
        self.region_tree.mark_widget_dirty(widget)?;

        if self.region_tree.is_dirty() {
            dirty_layers.insert(self.id);
        }

        Ok(())
    }

    pub fn set_widget_region_listens_to_pointer_events(
        &mut self,
        widget: &StrongWidgetEntry<MSG>,
        listens: bool,
    ) -> Result<(), ()> {
        self.region_tree
            .set_widget_listens_to_pointer_events(widget, listens)
    }

    pub fn handle_pointer_event(
        &mut self,
        mut event: PointerEvent,
        msg_out_queue: &mut Vec<MSG>,
    ) -> Option<(StrongWidgetEntry<MSG>, WidgetRequests)> {
        if !self.region_tree.layer_explicit_visibility() {
            return None;
        }

        if event.position.x < self.outer_position.x
            || event.position.y < self.outer_position.y
            || event.position.x > self.outer_position.x + self.region_tree.layer_size().width()
            || event.position.y > self.outer_position.y + self.region_tree.layer_size().height()
        {
            return None;
        }

        // Remove this layer's offset from the position of the mouse event.
        event.position -= self.outer_position;

        self.region_tree.handle_pointer_event(event, msg_out_queue)
    }

    pub fn is_empty(&self) -> bool {
        self.region_tree.is_empty()
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
