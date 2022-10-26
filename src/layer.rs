use std::cmp::Ordering;
use std::hash::Hash;

use crate::anchor::Anchor;
use crate::app_window::{StrongWidgetEntry, WidgetSet};
use crate::error::FirewheelError;
use crate::event::PointerEvent;
use crate::renderer::LayerRenderer;
use crate::size::{PhysicalPoint, Point, Size};
use crate::{ScaleFactor, WidgetRegionType, WidgetRequests};

mod region_tree;
use region_tree::RegionTree;
pub(crate) use region_tree::WeakRegionTreeEntry;
pub use region_tree::{ContainerRegionRef, ParentAnchorType, RegionInfo};

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
    pub renderer: Option<LayerRenderer>,

    pub region_tree: RegionTree<MSG>,
    pub outer_position: Point,
    pub physical_outer_position: PhysicalPoint,
}

impl<MSG> Layer<MSG> {
    pub fn new(
        id: LayerID,
        size: Size,
        outer_position: Point,
        inner_position: Point,
        explicit_visibility: bool,
        window_visibility: bool,
        scale_factor: ScaleFactor,
    ) -> Result<Self, FirewheelError> {
        Ok(Self {
            id,
            renderer: Some(LayerRenderer::new()),
            region_tree: RegionTree::new(
                size,
                inner_position,
                explicit_visibility,
                window_visibility,
                scale_factor,
                id,
            ),
            outer_position,
            physical_outer_position: outer_position.to_physical(scale_factor),
        })
    }

    pub fn set_outer_position(&mut self, position: Point, scale_factor: ScaleFactor) {
        self.outer_position = position;
        self.physical_outer_position = position.to_physical(scale_factor);
    }

    pub fn set_inner_position(
        &mut self,
        position: Point,
        widgets_just_shown: &mut WidgetSet<MSG>,
        widgets_just_hidden: &mut WidgetSet<MSG>,
    ) {
        self.region_tree.set_layer_inner_position(
            position,
            widgets_just_shown,
            widgets_just_hidden,
        );
    }

    pub fn set_explicit_visibility(
        &mut self,
        explicit_visibility: bool,
        widgets_just_shown: &mut WidgetSet<MSG>,
        widgets_just_hidden: &mut WidgetSet<MSG>,
    ) {
        self.region_tree.set_layer_explicit_visibility(
            explicit_visibility,
            widgets_just_shown,
            widgets_just_hidden,
        );
    }

    pub fn set_window_visibility(
        &mut self,
        visible: bool,
        widgets_just_shown: &mut WidgetSet<MSG>,
        widgets_just_hidden: &mut WidgetSet<MSG>,
    ) {
        self.region_tree
            .set_window_visibility(visible, widgets_just_shown, widgets_just_hidden);
    }

    pub fn set_size(
        &mut self,
        size: Size,
        scale_factor: ScaleFactor,
        widgets_just_shown: &mut WidgetSet<MSG>,
        widgets_just_hidden: &mut WidgetSet<MSG>,
    ) {
        self.region_tree.set_layer_size(
            size,
            scale_factor,
            widgets_just_shown,
            widgets_just_hidden,
        );
    }

    pub fn add_container_region(
        &mut self,
        region_info: RegionInfo<MSG>,
        explicit_visibility: bool,
        widgets_just_shown: &mut WidgetSet<MSG>,
        widgets_just_hidden: &mut WidgetSet<MSG>,
    ) -> Result<ContainerRegionRef<MSG>, FirewheelError> {
        self.region_tree.add_container_region(
            region_info,
            explicit_visibility,
            widgets_just_shown,
            widgets_just_hidden,
        )
    }

    pub fn remove_container_region(
        &mut self,
        container_ref: ContainerRegionRef<MSG>,
    ) -> Result<(), FirewheelError> {
        self.region_tree.remove_container_region(container_ref)
    }

    pub fn modify_container_region(
        &mut self,
        container_ref: &mut ContainerRegionRef<MSG>,
        new_size: Option<Size>,
        new_internal_anchor: Option<Anchor>,
        new_parent_anchor: Option<Anchor>,
        new_anchor_offset: Option<Point>,
        widgets_just_shown: &mut WidgetSet<MSG>,
        widgets_just_hidden: &mut WidgetSet<MSG>,
    ) -> Result<(), FirewheelError> {
        self.region_tree.modify_container_region(
            container_ref,
            new_size,
            new_internal_anchor,
            new_parent_anchor,
            new_anchor_offset,
            widgets_just_shown,
            widgets_just_hidden,
        )
    }

    pub fn set_container_region_explicit_visibility(
        &mut self,
        container_ref: &mut ContainerRegionRef<MSG>,
        visible: bool,
        widgets_just_shown: &mut WidgetSet<MSG>,
        widgets_just_hidden: &mut WidgetSet<MSG>,
    ) -> Result<(), FirewheelError> {
        self.region_tree.set_container_region_explicit_visibility(
            container_ref,
            visible,
            widgets_just_shown,
            widgets_just_hidden,
        )
    }

    pub fn mark_container_region_dirty(
        &mut self,
        container_ref: &mut ContainerRegionRef<MSG>,
    ) -> Result<(), FirewheelError> {
        self.region_tree.mark_container_region_dirty(container_ref)
    }

    pub fn add_widget_region(
        &mut self,
        assigned_widget: &mut StrongWidgetEntry<MSG>,
        region_info: RegionInfo<MSG>,
        region_type: WidgetRegionType,
        explicit_visibility: bool,
        widgets_just_shown: &mut WidgetSet<MSG>,
        widgets_just_hidden: &mut WidgetSet<MSG>,
    ) -> Result<(), FirewheelError> {
        self.region_tree.add_widget_region(
            assigned_widget,
            region_info,
            region_type,
            explicit_visibility,
            widgets_just_shown,
            widgets_just_hidden,
        )
    }

    pub fn remove_widget_region(
        &mut self,
        widget: &mut StrongWidgetEntry<MSG>,
        widgets_just_shown: &mut WidgetSet<MSG>,
        widgets_just_hidden: &mut WidgetSet<MSG>,
    ) {
        self.region_tree
            .remove_widget_region(widget, widgets_just_shown, widgets_just_hidden);
    }

    pub fn modify_widget_region(
        &mut self,
        widget: &mut StrongWidgetEntry<MSG>,
        new_size: Option<Size>,
        new_internal_anchor: Option<Anchor>,
        new_parent_anchor: Option<Anchor>,
        new_anchor_offset: Option<Point>,
        widgets_just_shown: &mut WidgetSet<MSG>,
        widgets_just_hidden: &mut WidgetSet<MSG>,
    ) {
        self.region_tree.modify_widget_region(
            widget,
            new_size,
            new_internal_anchor,
            new_parent_anchor,
            new_anchor_offset,
            widgets_just_shown,
            widgets_just_hidden,
        );
    }

    pub fn set_widget_explicit_visibility(
        &mut self,
        widget: &mut StrongWidgetEntry<MSG>,
        visible: bool,
        widgets_just_shown: &mut WidgetSet<MSG>,
        widgets_just_hidden: &mut WidgetSet<MSG>,
    ) {
        self.region_tree.set_widget_explicit_visibility(
            widget,
            visible,
            widgets_just_shown,
            widgets_just_hidden,
        );
    }

    pub fn mark_widget_region_dirty(&mut self, widget: &StrongWidgetEntry<MSG>) {
        self.region_tree.mark_widget_dirty(widget);
    }

    pub fn set_widget_region_listens_to_pointer_events(
        &mut self,
        widget: &StrongWidgetEntry<MSG>,
        listens: bool,
    ) {
        self.region_tree
            .set_widget_listens_to_pointer_events(widget, listens);
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

    pub fn is_dirty(&self) -> bool {
        self.region_tree.is_dirty()
    }

    pub fn is_visible(&self) -> bool {
        self.region_tree.is_visible()
    }

    pub fn size(&self) -> Size {
        self.region_tree.layer_size()
    }
}
