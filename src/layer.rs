use fnv::FnvHashSet;

use crate::anchor::Anchor;
use crate::canvas::SharedWidgetEntry;
use crate::event::{Event, MouseEvent};
use crate::size::{Point, Size};
use crate::{EventCapturedStatus, WidgetID, WidgetRequests};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::hash::Hash;

mod region_tree;
pub use region_tree::{ContainerRegionID, ParentAnchorType};
use region_tree::{RegionTree, SharedRegionTreeEntry};

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

struct RegionMouseState<MSG> {
    widget_id: WidgetID,
    widget: SharedWidgetEntry<MSG>,
    region: SharedRegionTreeEntry,
}

/// A set of widgets optimized for iteration.
struct WidgetRegionSet<MSG> {
    set: FnvHashSet<WidgetID>,
    entries: Vec<RegionMouseState<MSG>>,
}

impl<MSG> WidgetRegionSet<MSG> {
    fn new() -> Self {
        Self {
            set: FnvHashSet::default(),
            entries: Vec::new(),
        }
    }

    fn insert(
        &mut self,
        id: WidgetID,
        widget_entry: SharedWidgetEntry<MSG>,
        region_entry: SharedRegionTreeEntry,
    ) {
        if self.set.insert(id) {
            self.entries.push(RegionMouseState {
                widget_id: id,
                widget: widget_entry,
                region: region_entry,
            });
        } else {
            for entry in self.entries.iter_mut() {
                if entry.widget_id == id {
                    entry.widget = widget_entry;
                    entry.region = region_entry;
                    break;
                }
            }
        }
    }

    fn remove(&mut self, id: WidgetID) {
        if self.set.remove(&id) {
            let mut remove_i = None;
            for (i, entry) in self.entries.iter().enumerate() {
                if entry.widget_id == id {
                    remove_i = Some(i);
                    break;
                }
            }
            if let Some(i) = remove_i {
                self.entries.remove(i);
            }
        }
    }
}

pub(crate) struct Layer<MSG> {
    pub id: LayerID,

    region_tree: RegionTree,
    position: Point,

    size: Size,
    min_size: Option<Size>,
    max_size: Option<Size>,

    widgets_with_mouse_listen: WidgetRegionSet<MSG>,

    visible: bool,
}

impl<MSG> Layer<MSG> {
    pub fn new(
        id: LayerID,
        size: Size,
        min_size: Option<Size>,
        max_size: Option<Size>,
        position: Point,
    ) -> Result<Self, LayerError> {
        Self::check_min_max_size(min_size, max_size)?;
        let size = Self::clamp_size(size, min_size, max_size);

        Ok(Self {
            id,
            region_tree: RegionTree::new(size),
            position,
            size: size,
            min_size,
            max_size,
            widgets_with_mouse_listen: WidgetRegionSet::new(),
            visible: true,
        })
    }

    pub fn set_position(&mut self, position: Point) {
        self.position = position;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn modify(
        &mut self,
        new_size: Option<Size>,
        new_min_size: Option<Size>,
        new_max_size: Option<Size>,
        dirty_layers: &mut FnvHashSet<LayerID>,
    ) -> Result<(), LayerError> {
        Self::check_min_max_size(new_min_size, new_max_size)?;
        self.min_size = new_min_size;
        self.max_size = new_max_size;

        let size = Self::clamp_size(new_size.unwrap_or(self.size), new_min_size, new_max_size);

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
    ) -> Result<ContainerRegionID, ()> {
        self.region_tree.new_container_region(
            size,
            internal_anchor,
            parent_anchor,
            parent_anchor_type,
            anchor_offset,
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

    pub fn handle_mouse_event(
        &mut self,
        mut event: MouseEvent,
    ) -> Option<(WidgetID, WidgetRequests<MSG>)> {
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

        let mut captured = None;
        for region_mouse_state in self.widgets_with_mouse_listen.entries.iter_mut() {
            let region_rect = region_mouse_state.region.borrow().rect();
            if region_rect.contains_point(event.position) {
                // Remove the region's offset from the position of the mouse event.
                let temp_position = event.position;
                let temp_prev_position = event.previous_position;
                event.position -= region_rect.pos;
                event.previous_position -= region_rect.pos;

                if let EventCapturedStatus::Captured(requests) = region_mouse_state
                    .widget
                    .borrow_mut()
                    .on_event(&Event::Mouse(event))
                {
                    captured = Some((region_mouse_state.widget_id, requests));
                    break;
                }

                event.position = temp_position;
                event.previous_position = temp_prev_position;
            }
        }

        captured
    }

    fn check_min_max_size(
        min_size: Option<Size>,
        max_size: Option<Size>,
    ) -> Result<(), LayerError> {
        if let Some(min_size) = min_size {
            if let Some(max_size) = max_size {
                if min_size.width() > max_size.width() || min_size.height() > max_size.height() {
                    return Err(LayerError::MinSizeNotLessThanMaxSize { min_size, max_size });
                }
            }
        }
        Ok(())
    }

    fn clamp_size(mut size: Size, min_size: Option<Size>, max_size: Option<Size>) -> Size {
        if let Some(min_size) = min_size {
            size = size.max(min_size);
        }
        if let Some(max_size) = max_size {
            size = size.min(max_size);
        }
        size
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayerError {
    LayerWithIDNotFound(LayerID),
    MinSizeNotLessThanMaxSize { min_size: Size, max_size: Size },
}

impl Error for LayerError {}

impl fmt::Display for LayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LayerWithIDNotFound(id) => {
                write!(f, "Could not find layer with ID {:?}", id)
            }
            Self::MinSizeNotLessThanMaxSize { min_size, max_size } => {
                write!(
                    f,
                    "Layer has a minimum size {:?} that is not less than its maximum size {:?}",
                    min_size, max_size
                )
            }
        }
    }
}
