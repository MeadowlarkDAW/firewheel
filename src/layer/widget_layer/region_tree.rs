use std::cell::{RefCell, RefMut};
use std::rc::{Rc, Weak};

use crate::error::FirewheelError;
use crate::event::{InputEvent, PointerEvent};
use crate::layer::WeakWidgetLayerEntry;
use crate::node::StrongWidgetNodeEntry;
use crate::size::{PhysicalPoint, PhysicalRect, PhysicalSize, TextureRect};
use crate::widget_node_set::WidgetNodeSet;
use crate::{
    Anchor, EventCapturedStatus, HAlign, Point, Rect, ScaleFactor, Size, VAlign,
    WidgetNodeRequests, WidgetNodeType,
};

// TODO: Let the user specify whether child regions should be internally unsorted
// (default), sorted by x coordinate, or sorted by y coordinate. Sorted lists will
// allow for further scrolling and pointer input optimizations for long lists of
// items.

#[derive(Clone)]
pub struct RegionInfo<MSG> {
    pub size: Size,
    pub internal_anchor: Anchor,
    pub parent_anchor: Anchor,
    pub parent_anchor_type: ParentAnchorType<MSG>,
    pub anchor_offset: Point,
}

pub(crate) struct RegionTree<MSG> {
    pub dirty_widgets: WidgetNodeSet<MSG>,
    pub texture_rects_to_clear: Vec<TextureRect>,
    pub clear_whole_layer: bool,

    next_region_id: u64,
    roots: Vec<StrongRegionTreeEntry<MSG>>,
    layer_rect: Rect,
    layer_physical_rect: PhysicalRect,
    layer_explicit_visibility: bool,
    window_visibility: bool,
    scale_factor: ScaleFactor,
    layer_id: u64,
}

impl<MSG> RegionTree<MSG> {
    pub fn new(
        layer_size: Size,
        inner_position: Point,
        layer_explicit_visibility: bool,
        window_visibility: bool,
        scale_factor: ScaleFactor,
        layer_id: u64,
    ) -> Self {
        Self {
            next_region_id: 0,
            roots: Vec::new(),
            dirty_widgets: WidgetNodeSet::new(),
            texture_rects_to_clear: Vec::new(),
            layer_rect: Rect::new(inner_position, layer_size),
            layer_physical_rect: PhysicalRect::new(
                inner_position.to_physical(scale_factor),
                layer_size.to_physical(scale_factor),
            ),
            layer_explicit_visibility,
            window_visibility,
            clear_whole_layer: true,
            scale_factor,
            layer_id,
        }
    }

    pub fn add_container_region(
        &mut self,
        region_info: RegionInfo<MSG>,
        explicit_visibility: bool,
        widgets_just_shown: &mut WidgetNodeSet<MSG>,
        widgets_just_hidden: &mut WidgetNodeSet<MSG>,
    ) -> Result<ContainerRegionRef<MSG>, FirewheelError> {
        let new_id = self.next_region_id;
        self.next_region_id += 1;

        let mut new_entry = StrongRegionTreeEntry {
            shared: Rc::new(RefCell::new(RegionTreeEntry {
                region: Region {
                    id: new_id,
                    internal_anchor: region_info.internal_anchor,
                    parent_anchor: region_info.parent_anchor,
                    anchor_offset: region_info.anchor_offset,
                    rect: Rect::new(Point::default(), region_info.size), // The position will be overwritten
                    physical_rect: PhysicalRect::new(
                        PhysicalPoint::default(), // The position will be overwritten
                        region_info.size.to_physical(self.scale_factor),
                    ),
                    parent_rect: Rect::default(), // This will be overwritten
                    last_rendered_texture_rect: None,
                    explicit_visibility,
                    parent_explicit_visibility: false, // This will be overwritten
                    is_within_layer_rect: false,       // This will be overwritten
                    is_visible: false,                 // This will be overwritten
                },
                parent: None,
                children: Some(Vec::new()),
                assigned_widget: None,
            })),
            region_id: new_id,
        };

        let (parent_rect, parent_explicit_visibility) = match region_info.parent_anchor_type {
            ParentAnchorType::Layer => {
                self.roots.push(new_entry.clone());

                (
                    self.layer_rect,
                    self.layer_explicit_visibility && self.window_visibility,
                )
            }
            ParentAnchorType::ContainerRegion(container_ref) => {
                if container_ref.assigned_layer_id != self.layer_id {
                    return Err(FirewheelError::ParentAnchorRegionNotPartOfLayer);
                }

                let (parent_rect, parent_explicit_visibility) =
                    if let Some(parent_entry) = container_ref.shared.upgrade() {
                        let (parent_rect, parent_explicit_visibility) = {
                            let mut parent_entry_ref = parent_entry.borrow_mut();
                            if let Some(children) = &mut parent_entry_ref.children {
                                children.push(new_entry.clone());
                            } else {
                                panic!("Parent region is not a container region");
                            }
                            (
                                parent_entry_ref.region.rect,
                                parent_entry_ref.region.explicit_visibility
                                    && parent_entry_ref.region.parent_explicit_visibility
                                    && self.window_visibility,
                            )
                        };
                        {
                            new_entry.borrow_mut().parent = Some(container_ref.shared.clone());
                        }

                        (parent_rect, parent_explicit_visibility)
                    } else {
                        return Err(FirewheelError::ParentAnchorRegionRemoved);
                    };

                (parent_rect, parent_explicit_visibility)
            }
        };
        {
            new_entry.borrow_mut().parent_changed(
                parent_rect,
                self.layer_rect,
                self.scale_factor,
                parent_explicit_visibility,
                &mut self.dirty_widgets,
                &mut self.texture_rects_to_clear,
                widgets_just_shown,
                widgets_just_hidden,
            );
        }

        let container_ref = ContainerRegionRef {
            shared: new_entry.downgrade(),
            assigned_layer: WeakWidgetLayerEntry::new(), // This will be overwritten.
            assigned_layer_id: self.layer_id,
            _unique_id: new_id,
        };

        Ok(container_ref)
    }

    pub fn remove_container_region(
        &mut self,
        container_ref: ContainerRegionRef<MSG>,
    ) -> Result<(), FirewheelError> {
        if container_ref.assigned_layer_id != self.layer_id {
            panic!("container region was not assigned to this layer");
        }

        let entry = container_ref
            .shared
            .upgrade()
            .take()
            .ok_or_else(|| FirewheelError::ContainerRegionRemoved)?;
        let mut entry_ref = entry.borrow_mut();

        if let Some(children) = &entry_ref.children {
            if !children.is_empty() {
                return Err(FirewheelError::ContainerRegionNotEmpty);
            }
        } else {
            panic!("region was not a container region");
        }

        // Remove this child entry from its parent.
        if let Some(parent_entry) = entry_ref.parent.as_mut() {
            let parent_entry = parent_entry.upgrade().unwrap();
            let mut parent_entry = parent_entry.borrow_mut();

            if let Some(children) = &mut parent_entry.children {
                let mut remove_i = None;
                for (i, e) in children.iter().enumerate() {
                    if e.region_id == entry_ref.region.id {
                        remove_i = Some(i);
                        break;
                    }
                }
                if let Some(i) = remove_i {
                    children.remove(i);
                } else {
                    panic!("parent region did not contain child region");
                }
            } else {
                panic!("parent region was not a container region");
            }
        } else {
            // This entry had no parent, so remove it from the root entries instead.
            let mut remove_i = None;
            for (i, e) in self.roots.iter().enumerate() {
                if e.region_id == entry_ref.region.id {
                    remove_i = Some(i);
                    break;
                }
            }
            if let Some(i) = remove_i {
                self.roots.remove(i);
            } else {
                panic!("child region was not assigned to this layer");
            }
        }

        Ok(())
    }

    pub fn modify_container_region(
        &mut self,
        container_ref: &mut ContainerRegionRef<MSG>,
        new_size: Option<Size>,
        new_internal_anchor: Option<Anchor>,
        new_parent_anchor: Option<Anchor>,
        new_anchor_offset: Option<Point>,
        widgets_just_shown: &mut WidgetNodeSet<MSG>,
        widgets_just_hidden: &mut WidgetNodeSet<MSG>,
    ) -> Result<(), FirewheelError> {
        let entry = container_ref
            .shared
            .upgrade()
            .ok_or_else(|| FirewheelError::ContainerRegionRemoved)?;

        entry.borrow_mut().modify(
            new_size,
            new_internal_anchor,
            new_parent_anchor,
            new_anchor_offset,
            None,
            self.layer_rect,
            self.scale_factor,
            &mut self.dirty_widgets,
            &mut self.texture_rects_to_clear,
            widgets_just_shown,
            widgets_just_hidden,
        );

        Ok(())
    }

    pub fn mark_container_region_dirty(
        &mut self,
        container_ref: &mut ContainerRegionRef<MSG>,
    ) -> Result<(), FirewheelError> {
        let entry = container_ref
            .shared
            .upgrade()
            .ok_or_else(|| FirewheelError::ContainerRegionRemoved)?;

        entry
            .borrow_mut()
            .mark_dirty(&mut self.dirty_widgets, &mut self.texture_rects_to_clear);

        Ok(())
    }

    pub fn set_container_region_explicit_visibility(
        &mut self,
        container_ref: &mut ContainerRegionRef<MSG>,
        explicit_visibility: bool,
        widgets_just_shown: &mut WidgetNodeSet<MSG>,
        widgets_just_hidden: &mut WidgetNodeSet<MSG>,
    ) -> Result<(), FirewheelError> {
        let entry = container_ref
            .shared
            .upgrade()
            .ok_or_else(|| FirewheelError::ContainerRegionRemoved)?;

        entry.borrow_mut().modify(
            None,
            None,
            None,
            None,
            Some(explicit_visibility),
            self.layer_rect,
            self.scale_factor,
            &mut self.dirty_widgets,
            &mut self.texture_rects_to_clear,
            widgets_just_shown,
            widgets_just_hidden,
        );

        Ok(())
    }

    pub fn add_widget_region(
        &mut self,
        assigned_widget: &mut StrongWidgetNodeEntry<MSG>,
        region_info: RegionInfo<MSG>,
        node_type: WidgetNodeType,
        explicit_visibility: bool,
        widgets_just_shown: &mut WidgetNodeSet<MSG>,
        widgets_just_hidden: &mut WidgetNodeSet<MSG>,
    ) -> Result<(), FirewheelError> {
        if assigned_widget.assigned_region().upgrade().is_some() {
            panic!("widget was already assigned a region");
        }

        let new_id = self.next_region_id;
        self.next_region_id += 1;

        let mut new_entry = StrongRegionTreeEntry {
            shared: Rc::new(RefCell::new(RegionTreeEntry {
                region: Region {
                    id: new_id,
                    internal_anchor: region_info.internal_anchor,
                    parent_anchor: region_info.parent_anchor,
                    anchor_offset: region_info.anchor_offset,
                    rect: Rect::new(Point::default(), region_info.size), // This will be overwritten
                    physical_rect: PhysicalRect::new(
                        PhysicalPoint::default(), // The position will be overwritten
                        region_info.size.to_physical(self.scale_factor),
                    ),
                    parent_rect: Rect::default(), // This will be overwritten
                    last_rendered_texture_rect: None,
                    explicit_visibility,
                    parent_explicit_visibility: false, // This will be overwritten
                    is_within_layer_rect: false,       // This will be overwritten
                    is_visible: false,                 // This will be overwritten
                },
                parent: None,
                children: None,
                assigned_widget: Some(RegionAssignedWidget {
                    widget: assigned_widget.clone(),
                    listens_to_pointer_events: false,
                    node_type,
                }),
            })),
            region_id: new_id,
        };

        assigned_widget.set_assigned_region(new_entry.downgrade());

        let (parent_rect, parent_explicit_visibility) = match region_info.parent_anchor_type {
            ParentAnchorType::Layer => {
                self.roots.push(new_entry.clone());

                (
                    self.layer_rect,
                    self.layer_explicit_visibility && self.window_visibility,
                )
            }
            ParentAnchorType::ContainerRegion(container_ref) => {
                if container_ref.assigned_layer_id != self.layer_id {
                    return Err(FirewheelError::ParentAnchorRegionNotPartOfLayer);
                }

                let (parent_rect, parent_explicit_visibility) =
                    if let Some(parent_entry) = container_ref.shared.upgrade() {
                        let (parent_rect, parent_explicit_visibility) = {
                            let mut parent_entry_ref = parent_entry.borrow_mut();
                            if let Some(children) = &mut parent_entry_ref.children {
                                children.push(new_entry.clone());
                            } else {
                                panic!("Parent region is not a container region");
                            }
                            (
                                parent_entry_ref.region.rect,
                                parent_entry_ref.region.explicit_visibility
                                    && parent_entry_ref.region.parent_explicit_visibility
                                    && self.window_visibility,
                            )
                        };
                        {
                            new_entry.borrow_mut().parent = Some(container_ref.shared.clone());
                        }

                        (parent_rect, parent_explicit_visibility)
                    } else {
                        return Err(FirewheelError::ParentAnchorRegionRemoved);
                    };

                (parent_rect, parent_explicit_visibility)
            }
        };

        {
            let weak_entry = new_entry.downgrade();
            let mut entry_ref = new_entry.borrow_mut();

            entry_ref
                .assigned_widget
                .as_mut()
                .unwrap()
                .widget
                .set_assigned_region(weak_entry);

            entry_ref.parent_changed(
                parent_rect,
                self.layer_rect,
                self.scale_factor,
                parent_explicit_visibility,
                &mut self.dirty_widgets,
                &mut self.texture_rects_to_clear,
                widgets_just_shown,
                widgets_just_hidden,
            );
        }

        Ok(())
    }

    pub fn remove_widget_region(
        &mut self,
        widget: &mut StrongWidgetNodeEntry<MSG>,
        widgets_just_shown: &mut WidgetNodeSet<MSG>,
        widgets_just_hidden: &mut WidgetNodeSet<MSG>,
    ) {
        let entry = {
            if let Some(entry) = widget.assigned_region().upgrade() {
                entry
            } else {
                panic!("widget was not assigned a region");
            }
        };
        widget.assigned_region_mut().clear();

        let mut entry_ref = entry.borrow_mut();
        let entry_region_id = entry_ref.region.id;

        if entry_ref.children.is_some() {
            panic!("region was not a widget region");
        }

        self.dirty_widgets
            .remove(&entry_ref.assigned_widget.as_ref().unwrap().widget);
        if let Some(rect) = entry_ref.region.last_rendered_texture_rect.take() {
            self.texture_rects_to_clear.push(rect);
        }

        widgets_just_shown.remove(widget);
        widgets_just_hidden.remove(widget);

        // Remove this child entry from its parent.
        if let Some(parent_entry) = entry_ref.parent.as_mut() {
            let parent_entry = parent_entry.upgrade().unwrap();
            let mut parent_entry = parent_entry.borrow_mut();

            if let Some(children) = &mut parent_entry.children {
                let mut remove_i = None;
                for (i, e) in children.iter().enumerate() {
                    if e.region_id == entry_region_id {
                        remove_i = Some(i);
                        break;
                    }
                }
                if let Some(i) = remove_i {
                    children.remove(i);
                } else {
                    panic!("parent region did not contain child region");
                }
            } else {
                panic!("parent region was not a container region");
            }
        } else {
            // This entry had no parent, so remove it from the root entries instead.
            let mut remove_i = None;
            for (i, e) in self.roots.iter().enumerate() {
                if e.region_id == entry_region_id {
                    remove_i = Some(i);
                    break;
                }
            }
            if let Some(i) = remove_i {
                self.roots.remove(i);
            } else {
                panic!("widget region was not assigned to layer");
            }
        }
    }

    pub fn modify_widget_region(
        &mut self,
        widget: &StrongWidgetNodeEntry<MSG>,
        new_size: Option<Size>,
        new_internal_anchor: Option<Anchor>,
        new_parent_anchor: Option<Anchor>,
        new_anchor_offset: Option<Point>,
        widgets_just_shown: &mut WidgetNodeSet<MSG>,
        widgets_just_hidden: &mut WidgetNodeSet<MSG>,
    ) {
        widget
            .assigned_region()
            .upgrade()
            .expect("Widget was not assigned a region")
            .borrow_mut()
            .modify(
                new_size,
                new_internal_anchor,
                new_parent_anchor,
                new_anchor_offset,
                None,
                self.layer_rect,
                self.scale_factor,
                &mut self.dirty_widgets,
                &mut self.texture_rects_to_clear,
                widgets_just_shown,
                widgets_just_hidden,
            );
    }

    pub fn mark_widget_dirty(&mut self, widget: &StrongWidgetNodeEntry<MSG>) {
        widget
            .assigned_region()
            .upgrade()
            .expect("Widget was not assigned a region")
            .borrow_mut()
            .mark_dirty(&mut self.dirty_widgets, &mut self.texture_rects_to_clear);
    }

    pub fn set_widget_explicit_visibility(
        &mut self,
        widget: &StrongWidgetNodeEntry<MSG>,
        explicit_visibility: bool,
        widgets_just_shown: &mut WidgetNodeSet<MSG>,
        widgets_just_hidden: &mut WidgetNodeSet<MSG>,
    ) {
        widget
            .assigned_region()
            .upgrade()
            .expect("Widget was not assigned a region")
            .borrow_mut()
            .modify(
                None,
                None,
                None,
                None,
                Some(explicit_visibility),
                self.layer_rect,
                self.scale_factor,
                &mut self.dirty_widgets,
                &mut self.texture_rects_to_clear,
                widgets_just_shown,
                widgets_just_hidden,
            );
    }

    pub fn set_widget_listens_to_pointer_events(
        &mut self,
        widget: &StrongWidgetNodeEntry<MSG>,
        listens: bool,
    ) {
        widget
            .assigned_region()
            .upgrade()
            .expect("Widget was not assigned a region")
            .borrow_mut()
            .assigned_widget
            .as_mut()
            .unwrap()
            .listens_to_pointer_events = listens;
    }

    pub fn set_layer_inner_position(
        &mut self,
        position: Point,
        widgets_just_shown: &mut WidgetNodeSet<MSG>,
        widgets_just_hidden: &mut WidgetNodeSet<MSG>,
    ) {
        if self.layer_rect.pos() != position {
            self.layer_rect.set_pos(position);
            self.layer_physical_rect.pos = self.layer_rect.pos().to_physical(self.scale_factor);
            self.clear_whole_layer = true;

            for entry in self.roots.iter_mut() {
                entry.borrow_mut().parent_changed(
                    self.layer_rect,
                    self.layer_rect,
                    self.scale_factor,
                    self.layer_explicit_visibility,
                    &mut self.dirty_widgets,
                    &mut self.texture_rects_to_clear,
                    widgets_just_shown,
                    widgets_just_hidden,
                );
            }
        }
    }

    pub fn set_layer_size(
        &mut self,
        size: Size,
        scale_factor: ScaleFactor,
        widgets_just_shown: &mut WidgetNodeSet<MSG>,
        widgets_just_hidden: &mut WidgetNodeSet<MSG>,
    ) {
        if self.layer_rect.size() != size || self.scale_factor != scale_factor {
            self.layer_rect.set_size(size);
            self.layer_physical_rect.size = self.layer_rect.size().to_physical(scale_factor);
            self.scale_factor = scale_factor;
            self.clear_whole_layer = true;

            for entry in self.roots.iter_mut() {
                entry.borrow_mut().parent_changed(
                    self.layer_rect,
                    self.layer_rect,
                    self.scale_factor,
                    self.layer_explicit_visibility,
                    &mut self.dirty_widgets,
                    &mut self.texture_rects_to_clear,
                    widgets_just_shown,
                    widgets_just_hidden,
                );
            }
        }
    }

    pub fn set_layer_explicit_visibility(
        &mut self,
        explicit_visibility: bool,
        widgets_just_shown: &mut WidgetNodeSet<MSG>,
        widgets_just_hidden: &mut WidgetNodeSet<MSG>,
    ) {
        if self.layer_explicit_visibility != explicit_visibility {
            self.layer_explicit_visibility = explicit_visibility;
            self.clear_whole_layer = true;

            for entry in self.roots.iter_mut() {
                entry.borrow_mut().parent_changed(
                    self.layer_rect,
                    self.layer_rect,
                    self.scale_factor,
                    self.layer_explicit_visibility,
                    &mut self.dirty_widgets,
                    &mut self.texture_rects_to_clear,
                    widgets_just_shown,
                    widgets_just_hidden,
                );
            }
        }
    }

    pub fn set_window_visibility(
        &mut self,
        visible: bool,
        widgets_just_shown: &mut WidgetNodeSet<MSG>,
        widgets_just_hidden: &mut WidgetNodeSet<MSG>,
    ) {
        self.window_visibility = visible;

        if self.is_visible() {
            let parent_explicit_visibility =
                self.window_visibility && self.layer_explicit_visibility;

            for entry in self.roots.iter_mut() {
                entry.borrow_mut().parent_changed(
                    self.layer_rect,
                    self.layer_rect,
                    self.scale_factor,
                    parent_explicit_visibility,
                    &mut self.dirty_widgets,
                    &mut self.texture_rects_to_clear,
                    widgets_just_shown,
                    widgets_just_hidden,
                );
            }
        }
    }

    pub fn layer_explicit_visibility(&self) -> bool {
        self.layer_explicit_visibility
    }

    pub fn layer_size(&self) -> Size {
        self.layer_rect.size()
    }

    pub fn layer_physical_size(&self) -> PhysicalSize {
        self.layer_physical_rect.size
    }

    pub fn layer_physical_internal_offset(&self) -> PhysicalPoint {
        self.layer_physical_rect.pos
    }

    pub fn layer_rect(&self) -> Rect {
        self.layer_rect
    }

    pub fn is_dirty(&self) -> bool {
        !self.dirty_widgets.is_empty()
            || !self.texture_rects_to_clear.is_empty()
            || self.clear_whole_layer
    }

    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }

    pub fn is_visible(&self) -> bool {
        self.layer_explicit_visibility && !self.roots.is_empty()
    }

    pub fn handle_pointer_event(
        &mut self,
        mut event: PointerEvent,
        msg_out_queue: &mut Vec<MSG>,
    ) -> Option<(StrongWidgetNodeEntry<MSG>, WidgetNodeRequests)> {
        if !self.layer_explicit_visibility {
            return None;
        }

        // Add this layer's inner position to the position of the pointer.
        event.position += self.layer_rect.pos();

        for region in self.roots.iter_mut() {
            match region
                .borrow_mut()
                .handle_pointer_event(event, msg_out_queue)
            {
                PointerCapturedStatus::Captured { widget, requests } => {
                    return Some((widget, requests));
                }
                PointerCapturedStatus::InRegionButNotCaptured => {
                    return None;
                }
                PointerCapturedStatus::NotInRegion => {}
            }
        }

        None
    }
}

struct StrongRegionTreeEntry<MSG> {
    shared: Rc<RefCell<RegionTreeEntry<MSG>>>,
    region_id: u64,
}

impl<MSG> StrongRegionTreeEntry<MSG> {
    fn borrow_mut(&mut self) -> RefMut<'_, RegionTreeEntry<MSG>> {
        RefCell::borrow_mut(&self.shared)
    }

    fn downgrade(&self) -> WeakRegionTreeEntry<MSG> {
        WeakRegionTreeEntry {
            shared: Rc::downgrade(&self.shared),
            region_id: self.region_id,
        }
    }
}

impl<MSG> Clone for StrongRegionTreeEntry<MSG> {
    fn clone(&self) -> Self {
        Self {
            shared: Rc::clone(&self.shared),
            region_id: self.region_id,
        }
    }
}

pub(crate) struct WeakRegionTreeEntry<MSG> {
    shared: Weak<RefCell<RegionTreeEntry<MSG>>>,
    region_id: u64,
}

impl<MSG> WeakRegionTreeEntry<MSG> {
    pub fn new() -> Self {
        Self {
            shared: Weak::new(),
            region_id: u64::MAX,
        }
    }

    pub fn upgrade(&self) -> Option<Rc<RefCell<RegionTreeEntry<MSG>>>> {
        self.shared.upgrade()
    }

    pub fn clear(&mut self) {
        self.shared = Weak::new();
    }
}

impl<MSG> Clone for WeakRegionTreeEntry<MSG> {
    fn clone(&self) -> Self {
        Self {
            shared: Weak::clone(&self.shared),
            region_id: self.region_id,
        }
    }
}

enum PointerCapturedStatus<MSG> {
    Captured {
        widget: StrongWidgetNodeEntry<MSG>,
        requests: WidgetNodeRequests,
    },
    InRegionButNotCaptured,
    NotInRegion,
}

struct RegionAssignedWidget<MSG> {
    widget: StrongWidgetNodeEntry<MSG>,
    listens_to_pointer_events: bool,
    node_type: WidgetNodeType,
}

pub(crate) struct RegionTreeEntry<MSG> {
    pub region: Region,
    parent: Option<WeakRegionTreeEntry<MSG>>,
    children: Option<Vec<StrongRegionTreeEntry<MSG>>>,
    assigned_widget: Option<RegionAssignedWidget<MSG>>,
}

impl<MSG> RegionTreeEntry<MSG> {
    fn handle_pointer_event(
        &mut self,
        mut event: PointerEvent,
        msg_out_queue: &mut Vec<MSG>,
    ) -> PointerCapturedStatus<MSG> {
        if self.region.is_visible() {
            if let Some(assigned_widget) = &mut self.assigned_widget {
                if assigned_widget.listens_to_pointer_events {
                    if self.region.rect.contains_point(event.position) {
                        // Remove the region's offset from the position of the mouse event.
                        let temp_position = event.position;
                        event.position -= self.region.rect.pos();

                        let status = {
                            assigned_widget
                                .widget
                                .borrow_mut()
                                .on_input_event(&InputEvent::Pointer(event), msg_out_queue)
                        };
                        let status = if let EventCapturedStatus::Captured(requests) = status {
                            PointerCapturedStatus::Captured {
                                widget: assigned_widget.widget.clone(),
                                requests,
                            }
                        } else {
                            PointerCapturedStatus::InRegionButNotCaptured
                        };

                        event.position = temp_position;

                        return status;
                    }
                }
            } else if self.region.rect.contains_point(event.position) {
                if let Some(children) = &mut self.children {
                    for child_region in children.iter_mut() {
                        match child_region
                            .borrow_mut()
                            .handle_pointer_event(event, msg_out_queue)
                        {
                            PointerCapturedStatus::Captured { widget, requests } => {
                                return PointerCapturedStatus::Captured { widget, requests };
                            }
                            PointerCapturedStatus::InRegionButNotCaptured => {
                                return PointerCapturedStatus::InRegionButNotCaptured;
                            }
                            PointerCapturedStatus::NotInRegion => {}
                        }
                    }
                }

                return PointerCapturedStatus::InRegionButNotCaptured;
            }
        }

        PointerCapturedStatus::NotInRegion
    }

    fn mark_dirty(
        &mut self,
        dirty_widgets: &mut WidgetNodeSet<MSG>,
        texture_rects_to_clear: &mut Vec<TextureRect>,
    ) {
        if self.region.is_visible() {
            if let Some(assigned_widget_info) = &self.assigned_widget {
                if let WidgetNodeType::Painted = assigned_widget_info.node_type {
                    dirty_widgets.insert(&assigned_widget_info.widget);
                    if let Some(rect) = self.region.last_rendered_texture_rect.take() {
                        texture_rects_to_clear.push(rect);
                    }
                }
            } else if let Some(children) = &mut self.children {
                for child_entry in children.iter_mut() {
                    child_entry
                        .borrow_mut()
                        .mark_dirty(dirty_widgets, texture_rects_to_clear);
                }
            }
        }
    }

    fn modify(
        &mut self,
        new_size: Option<Size>,
        new_internal_anchor: Option<Anchor>,
        new_parent_anchor: Option<Anchor>,
        new_anchor_offset: Option<Point>,
        new_explicit_visibility: Option<bool>,
        layer_rect: Rect,
        scale_factor: ScaleFactor,
        dirty_widgets: &mut WidgetNodeSet<MSG>,
        texture_rects_to_clear: &mut Vec<TextureRect>,
        widgets_just_shown: &mut WidgetNodeSet<MSG>,
        widgets_just_hidden: &mut WidgetNodeSet<MSG>,
    ) {
        let mut changed = false;
        if let Some(new_size) = new_size {
            if self.region.rect.size() != new_size {
                self.region.rect.set_size(new_size);
                changed = true;
            }
        }
        if let Some(new_internal_anchor) = new_internal_anchor {
            if self.region.internal_anchor != new_internal_anchor {
                self.region.internal_anchor = new_internal_anchor;
                changed = true;
            }
        }
        if let Some(new_parent_anchor) = new_parent_anchor {
            if self.region.parent_anchor != new_parent_anchor {
                self.region.parent_anchor = new_parent_anchor;
                changed = true;
            }
        }
        if let Some(new_anchor_offset) = new_anchor_offset {
            if self.region.anchor_offset != new_anchor_offset {
                self.region.anchor_offset = new_anchor_offset;
                changed = true;
            }
        }
        if let Some(new_explicit_visibility) = new_explicit_visibility {
            if self.region.explicit_visibility != new_explicit_visibility {
                self.region.explicit_visibility = new_explicit_visibility;
                changed = true;
            }
        }

        if changed {
            self.region.update_rect(scale_factor);
            self.region.is_within_layer_rect = layer_rect.overlaps_with_rect(self.region.rect);
            let visibility_changed_to = self.region.sync_visibility();

            if let Some(assigned_widget_info) = &self.assigned_widget {
                if let Some(new_visibility) = visibility_changed_to {
                    if new_visibility {
                        widgets_just_shown.insert(&assigned_widget_info.widget);
                        widgets_just_hidden.remove(&assigned_widget_info.widget);

                        if let WidgetNodeType::Painted = assigned_widget_info.node_type {
                            dirty_widgets.insert(&assigned_widget_info.widget);
                            if let Some(rect) = self.region.last_rendered_texture_rect.take() {
                                texture_rects_to_clear.push(rect);
                            }
                        }
                    } else {
                        widgets_just_hidden.insert(&assigned_widget_info.widget);
                        widgets_just_shown.remove(&assigned_widget_info.widget);

                        if let WidgetNodeType::Painted = assigned_widget_info.node_type {
                            dirty_widgets.remove(&assigned_widget_info.widget);
                            if let Some(rect) = self.region.last_rendered_texture_rect.take() {
                                texture_rects_to_clear.push(rect);
                            }
                        }
                    }
                } else if self.region.is_visible() {
                    if let WidgetNodeType::Painted = assigned_widget_info.node_type {
                        // Mark the region as dirty since it has changed.
                        dirty_widgets.insert(&assigned_widget_info.widget);
                        if let Some(rect) = self.region.last_rendered_texture_rect.take() {
                            texture_rects_to_clear.push(rect);
                        }
                    }
                }
            } else if let Some(children) = &mut self.children {
                for child_entry in children.iter_mut() {
                    child_entry.borrow_mut().parent_changed(
                        self.region.rect,
                        layer_rect,
                        scale_factor,
                        self.region.explicit_visibility && self.region.parent_explicit_visibility,
                        dirty_widgets,
                        texture_rects_to_clear,
                        widgets_just_shown,
                        widgets_just_hidden,
                    );
                }
            }
        }
    }

    fn parent_changed(
        &mut self,
        parent_rect: Rect,
        layer_rect: Rect,
        scale_factor: ScaleFactor,
        parent_explicit_visibility: bool,
        dirty_widgets: &mut WidgetNodeSet<MSG>,
        texture_rects_to_clear: &mut Vec<TextureRect>,
        widgets_just_shown: &mut WidgetNodeSet<MSG>,
        widgets_just_hidden: &mut WidgetNodeSet<MSG>,
    ) {
        self.region.update_parent_rect(parent_rect, scale_factor);
        self.region.parent_explicit_visibility = parent_explicit_visibility;
        self.region.is_within_layer_rect = layer_rect.overlaps_with_rect(self.region.rect);
        let visibility_changed_to = self.region.sync_visibility();

        if let Some(assigned_widget_info) = &self.assigned_widget {
            if let Some(new_visibility) = visibility_changed_to {
                if new_visibility {
                    widgets_just_shown.insert(&assigned_widget_info.widget);
                    widgets_just_hidden.remove(&assigned_widget_info.widget);

                    if let WidgetNodeType::Painted = assigned_widget_info.node_type {
                        dirty_widgets.insert(&assigned_widget_info.widget);
                        if let Some(rect) = self.region.last_rendered_texture_rect.take() {
                            texture_rects_to_clear.push(rect);
                        }
                    }
                } else {
                    widgets_just_hidden.insert(&assigned_widget_info.widget);
                    widgets_just_shown.remove(&assigned_widget_info.widget);

                    if let WidgetNodeType::Painted = assigned_widget_info.node_type {
                        dirty_widgets.remove(&assigned_widget_info.widget);
                        if let Some(rect) = self.region.last_rendered_texture_rect.take() {
                            texture_rects_to_clear.push(rect);
                        }
                    }
                }
            } else if self.region.is_visible() {
                if let WidgetNodeType::Painted = assigned_widget_info.node_type {
                    // Mark the region as dirty as it likely moved because of the
                    // change to the parent rect (or the scale factor has changed).
                    dirty_widgets.insert(&assigned_widget_info.widget);
                    if let Some(rect) = self.region.last_rendered_texture_rect.take() {
                        texture_rects_to_clear.push(rect);
                    }
                }
            }
        } else if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                child.borrow_mut().parent_changed(
                    self.region.rect,
                    layer_rect,
                    scale_factor,
                    self.region.explicit_visibility && self.region.parent_explicit_visibility,
                    dirty_widgets,
                    texture_rects_to_clear,
                    widgets_just_shown,
                    widgets_just_hidden,
                );
            }
        }
    }
}

#[derive(Clone)]
pub struct ContainerRegionRef<MSG> {
    pub(crate) shared: WeakRegionTreeEntry<MSG>,
    pub(crate) assigned_layer: WeakWidgetLayerEntry<MSG>,
    assigned_layer_id: u64,
    _unique_id: u64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Region {
    pub id: u64,
    pub rect: Rect,
    pub physical_rect: PhysicalRect,
    pub internal_anchor: Anchor,
    pub parent_anchor: Anchor,
    pub anchor_offset: Point,
    pub last_rendered_texture_rect: Option<TextureRect>,
    pub parent_rect: Rect,
    pub explicit_visibility: bool,
    pub parent_explicit_visibility: bool,
    pub is_within_layer_rect: bool,
    is_visible: bool,
}

impl Region {
    fn update_rect(&mut self, scale_factor: ScaleFactor) {
        self.update_parent_rect(self.parent_rect, scale_factor);
    }

    fn update_parent_rect(&mut self, parent_rect: Rect, scale_factor: ScaleFactor) {
        let parent_anchor_pos_x = match self.parent_anchor.h_align {
            HAlign::Left => parent_rect.x(),
            HAlign::Center => parent_rect.center_x(),
            HAlign::Right => parent_rect.x2(),
        };
        let parent_anchor_pos_y = match self.parent_anchor.v_align {
            VAlign::Top => parent_rect.y(),
            VAlign::Center => parent_rect.center_y(),
            VAlign::Bottom => parent_rect.y2(),
        };

        self.parent_rect = parent_rect;

        let internal_anchor_pos_x = parent_anchor_pos_x + self.anchor_offset.x;
        let internal_anchor_pos_y = parent_anchor_pos_y + self.anchor_offset.y;

        let new_x = match self.internal_anchor.h_align {
            HAlign::Left => internal_anchor_pos_x,
            HAlign::Center => internal_anchor_pos_x - (self.rect.width() / 2.0),
            HAlign::Right => internal_anchor_pos_x - self.rect.width(),
        };
        let new_y = match self.internal_anchor.v_align {
            VAlign::Top => internal_anchor_pos_y,
            VAlign::Center => internal_anchor_pos_y - (self.rect.height() / 2.0),
            VAlign::Bottom => internal_anchor_pos_y - self.rect.height(),
        };

        self.rect.set_pos(Point::new(new_x, new_y));
        self.physical_rect = self.rect.to_physical(scale_factor);
    }

    pub fn sync_visibility(&mut self) -> Option<bool> {
        let old_visibility = self.is_visible;

        self.is_visible = self.explicit_visibility
            && self.parent_explicit_visibility
            && self.is_within_layer_rect;

        if self.is_visible != old_visibility {
            Some(self.is_visible)
        } else {
            None
        }
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
}

#[derive(Clone)]
pub enum ParentAnchorType<MSG> {
    Layer,
    ContainerRegion(ContainerRegionRef<MSG>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WidgetNode, WidgetNodeType};
    use std::cell::Ref;

    impl Region {
        fn new_test_region(
            id: u64,
            rect: Rect,
            physical_rect: PhysicalRect,
            region_info: RegionInfo<()>,
            last_rendered_texture_rect: Option<TextureRect>,
            parent_rect: Rect,
            explicit_visibility: bool,
            parent_explicit_visibility: bool,
            is_within_layer_rect: bool,
        ) -> Self {
            Self {
                id,
                rect,
                physical_rect,
                internal_anchor: region_info.internal_anchor,
                parent_anchor: region_info.parent_anchor,
                anchor_offset: region_info.anchor_offset,
                last_rendered_texture_rect,
                parent_rect,
                explicit_visibility,
                parent_explicit_visibility,
                is_within_layer_rect,
                is_visible: explicit_visibility & parent_explicit_visibility & is_within_layer_rect,
            }
        }
    }

    impl<MSG> StrongRegionTreeEntry<MSG> {
        fn borrow(&self) -> Ref<'_, RegionTreeEntry<MSG>> {
            RefCell::borrow(&self.shared)
        }
    }

    struct EmptyPaintedTestWidget {
        id: u64,
    }

    impl WidgetNode<()> for EmptyPaintedTestWidget {
        fn on_added(&mut self, _msg_out_queue: &mut Vec<()>) -> WidgetNodeType {
            println!("empty painted test widget {} added", self.id);
            WidgetNodeType::Painted
        }

        #[allow(unused)]
        fn on_removed(&mut self, _msg_out_queue: &mut Vec<()>) {
            println!("empty painted test widget {} remove", self.id);
        }

        fn on_input_event(
            &mut self,
            event: &InputEvent,
            _msg_out_queue: &mut Vec<()>,
        ) -> EventCapturedStatus {
            println!(
                "empty painted test widget {} got input event {:?}",
                self.id, event
            );
            EventCapturedStatus::NotCaptured
        }
    }

    struct EmptyPointerOnlyTestWidget {
        id: u64,
    }

    impl WidgetNode<()> for EmptyPointerOnlyTestWidget {
        fn on_added(&mut self, _msg_out_queue: &mut Vec<()>) -> WidgetNodeType {
            println!("empty pointer only test widget {} added", self.id);
            WidgetNodeType::PointerOnly
        }

        #[allow(unused)]
        fn on_removed(&mut self, _msg_out_queue: &mut Vec<()>) {
            println!("empty pointer only test widget {} remove", self.id);
        }

        fn on_input_event(
            &mut self,
            event: &InputEvent,
            _msg_out_queue: &mut Vec<()>,
        ) -> EventCapturedStatus {
            println!(
                "empty pointer only test widget {} got input event {:?}",
                self.id, event
            );
            EventCapturedStatus::NotCaptured
        }
    }

    #[test]
    fn test_region_tree() {
        let layer_rect = Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 100.0));
        let layer_explicit_visibility = true;
        let scale_factor = ScaleFactor(1.0);

        let mut widgets_just_shown: WidgetNodeSet<()> = WidgetNodeSet::new();
        let mut widgets_just_hidden: WidgetNodeSet<()> = WidgetNodeSet::new();

        let mut region_tree: RegionTree<()> = RegionTree::new(
            layer_rect.size(),
            layer_rect.pos(),
            true,
            true,
            scale_factor,
            0,
        );

        // --- Test adding container regions ----------------------------------------------------------
        // --------------------------------------------------------------------------------------------

        // container_root0: Tests the case of adding a container region that is
        // explicitly visible and within the layer bounds.
        let container_root0_region_info = RegionInfo {
            size: Size::new(100.0, 50.0),
            internal_anchor: Anchor {
                h_align: HAlign::Left,
                v_align: VAlign::Top,
            },
            parent_anchor: Anchor {
                h_align: HAlign::Left,
                v_align: VAlign::Top,
            },
            parent_anchor_type: ParentAnchorType::Layer,
            anchor_offset: Point::new(20.0, 10.0),
        };
        let container_root0_explicit_visibility = true;
        let container_root0_ref = region_tree
            .add_container_region(
                container_root0_region_info.clone(),
                container_root0_explicit_visibility,
                &mut widgets_just_shown,
                &mut widgets_just_hidden,
            )
            .unwrap();
        let container_root0_expected_rect = Rect::new(
            container_root0_region_info.anchor_offset,
            container_root0_region_info.size,
        );
        assert_region(
            &region_tree.roots[0].borrow().region,
            &Region::new_test_region(
                container_root0_ref._unique_id,
                container_root0_expected_rect,
                container_root0_expected_rect.to_physical(scale_factor),
                container_root0_region_info,
                None,
                layer_rect,
                container_root0_explicit_visibility,
                layer_explicit_visibility,
                true,
            ),
        );

        // container_root1: Tests the case of adding a container region that is
        // explicitly invisible and within the layer bounds.
        let container_root1_region_info = RegionInfo {
            size: Size::new(40.0, 50.0),
            internal_anchor: Anchor {
                h_align: HAlign::Right,
                v_align: VAlign::Bottom,
            },
            parent_anchor: Anchor {
                h_align: HAlign::Right,
                v_align: VAlign::Bottom,
            },
            parent_anchor_type: ParentAnchorType::Layer,
            anchor_offset: Point::new(-20.0, -10.0),
        };
        let container_root1_explicit_visibility = false;
        let container_root1_ref = region_tree
            .add_container_region(
                container_root1_region_info.clone(),
                container_root1_explicit_visibility,
                &mut widgets_just_shown,
                &mut widgets_just_hidden,
            )
            .unwrap();
        let container_root1_expected_rect = Rect::new(
            Point {
                x: layer_rect.x2() + container_root1_region_info.anchor_offset.x
                    - container_root1_region_info.size.width(),
                y: layer_rect.y2() + container_root1_region_info.anchor_offset.y
                    - container_root1_region_info.size.height(),
            },
            container_root1_region_info.size,
        );
        assert_region(
            &region_tree.roots[1].borrow().region,
            &Region::new_test_region(
                container_root1_ref._unique_id,
                container_root1_expected_rect,
                container_root1_expected_rect.to_physical(scale_factor),
                container_root1_region_info,
                None,
                layer_rect,
                container_root1_explicit_visibility,
                layer_explicit_visibility,
                true,
            ),
        );

        // container_root2: Tests the case of adding a container region that is
        // explicitly visible but not within the layer bounds.
        let container_root2_region_info = RegionInfo {
            size: Size::new(40.0, 50.0),
            internal_anchor: Anchor {
                h_align: HAlign::Left,
                v_align: VAlign::Top,
            },
            parent_anchor: Anchor {
                h_align: HAlign::Right,
                v_align: VAlign::Bottom,
            },
            parent_anchor_type: ParentAnchorType::Layer,
            anchor_offset: Point::new(100.0, 100.0),
        };
        let container_root2_explicit_visibility = true;
        let container_root2_ref = region_tree
            .add_container_region(
                container_root2_region_info.clone(),
                container_root2_explicit_visibility,
                &mut widgets_just_shown,
                &mut widgets_just_hidden,
            )
            .unwrap();
        let container_root2_expected_rect = Rect::new(
            Point {
                x: layer_rect.x2() + container_root2_region_info.anchor_offset.x,
                y: layer_rect.y2() + container_root2_region_info.anchor_offset.y,
            },
            container_root2_region_info.size,
        );
        assert_region(
            &region_tree.roots[2].borrow().region,
            &Region::new_test_region(
                container_root2_ref._unique_id,
                container_root2_expected_rect,
                container_root2_expected_rect.to_physical(scale_factor),
                container_root2_region_info,
                None,
                layer_rect,
                container_root2_explicit_visibility,
                layer_explicit_visibility,
                false,
            ),
        );

        // container_root3: Tests the case of adding a container region that is
        // explicitly invisible and not within the layer bounds.
        let container_root3_region_info = RegionInfo {
            size: Size::new(40.0, 50.0),
            internal_anchor: Anchor {
                h_align: HAlign::Left,
                v_align: VAlign::Top,
            },
            parent_anchor: Anchor {
                h_align: HAlign::Right,
                v_align: VAlign::Top,
            },
            parent_anchor_type: ParentAnchorType::Layer,
            anchor_offset: Point::new(300.0, 100.0),
        };
        let container_root3_explicit_visibility = false;
        let container_root3_ref = region_tree
            .add_container_region(
                container_root3_region_info.clone(),
                container_root3_explicit_visibility,
                &mut widgets_just_shown,
                &mut widgets_just_hidden,
            )
            .unwrap();
        let container_root3_expected_rect = Rect::new(
            Point {
                x: layer_rect.x2() + container_root3_region_info.anchor_offset.x,
                y: layer_rect.y() + container_root3_region_info.anchor_offset.y,
            },
            container_root3_region_info.size,
        );
        assert_region(
            &region_tree.roots[3].borrow().region,
            &Region::new_test_region(
                container_root3_ref._unique_id,
                container_root3_expected_rect,
                container_root3_expected_rect.to_physical(scale_factor),
                container_root3_region_info,
                None,
                layer_rect,
                container_root3_explicit_visibility,
                layer_explicit_visibility,
                false,
            ),
        );

        // container_root0_0: Tests the case of adding a container region that is
        // a child of another container region.
        let container_root0_0_region_info = RegionInfo {
            size: Size::new(50.0, 40.0),
            internal_anchor: Anchor {
                h_align: HAlign::Center,
                v_align: VAlign::Center,
            },
            parent_anchor: Anchor {
                h_align: HAlign::Center,
                v_align: VAlign::Center,
            },
            parent_anchor_type: ParentAnchorType::ContainerRegion(container_root0_ref.clone()),
            anchor_offset: Point::new(-10.0, 4.0),
        };
        let container_root0_0_explicit_visibility = true;
        let container_root0_0_ref = region_tree
            .add_container_region(
                container_root0_0_region_info.clone(),
                container_root0_0_explicit_visibility,
                &mut widgets_just_shown,
                &mut widgets_just_hidden,
            )
            .unwrap();
        let container_root0_0_expected_rect = Rect::new(
            Point {
                x: container_root0_expected_rect.center_x()
                    - (container_root0_0_region_info.size.width() / 2.0)
                    + container_root0_0_region_info.anchor_offset.x,
                y: container_root0_expected_rect.center_y()
                    - (container_root0_0_region_info.size.height() / 2.0)
                    + container_root0_0_region_info.anchor_offset.y,
            },
            container_root0_0_region_info.size,
        );
        assert_region(
            &region_tree.roots[0].borrow().children.as_ref().unwrap()[0]
                .borrow()
                .region,
            &Region::new_test_region(
                container_root0_0_ref._unique_id,
                container_root0_0_expected_rect,
                container_root0_0_expected_rect.to_physical(scale_factor),
                container_root0_0_region_info,
                None,
                container_root0_expected_rect,
                container_root0_0_explicit_visibility,
                layer_explicit_visibility && container_root0_explicit_visibility,
                true,
            ),
        );

        // These should all be empty because we haven't added any widget
        // regions yet.
        assert!(region_tree.texture_rects_to_clear.is_empty());
        assert!(region_tree.dirty_widgets.is_empty());
        assert!(widgets_just_shown.is_empty());
        assert!(widgets_just_hidden.is_empty());

        // --- Test adding widget regions -------------------------------------------------------------
        // --------------------------------------------------------------------------------------------

        // widget_root4: Tests the case of adding a widget region at root
        // level that is explicitly visible and within layer bounds.
        let mut widget_root4_entry = StrongWidgetNodeEntry::new(
            Rc::new(RefCell::new(Box::new(EmptyPaintedTestWidget { id: 0 }))),
            WeakWidgetLayerEntry::new(),
            WeakRegionTreeEntry::new(),
            0,
        );
        let widget_root4_region_info = RegionInfo {
            size: Size::new(10.0, 8.0),
            internal_anchor: Anchor {
                h_align: HAlign::Left,
                v_align: VAlign::Top,
            },
            parent_anchor: Anchor {
                h_align: HAlign::Left,
                v_align: VAlign::Top,
            },
            parent_anchor_type: ParentAnchorType::Layer,
            anchor_offset: Point::new(20.0, 40.0),
        };
        let widget_root4_explicit_visibility = true;
        region_tree
            .add_widget_region(
                &mut widget_root4_entry,
                widget_root4_region_info.clone(),
                WidgetNodeType::Painted,
                widget_root4_explicit_visibility,
                &mut widgets_just_shown,
                &mut widgets_just_hidden,
            )
            .unwrap();
        let widget_root4_expected_rect = Rect::new(
            widget_root4_region_info.anchor_offset,
            widget_root4_region_info.size,
        );
        assert_region(
            &region_tree.roots[4].borrow().region,
            &Region::new_test_region(
                widget_root4_entry
                    .assigned_region()
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .region
                    .id,
                widget_root4_expected_rect,
                widget_root4_expected_rect.to_physical(scale_factor),
                widget_root4_region_info,
                None,
                layer_rect,
                widget_root4_explicit_visibility,
                layer_explicit_visibility,
                true,
            ),
        );
        assert!(region_tree.dirty_widgets.contains(&widget_root4_entry));
        assert!(widgets_just_shown.contains(&widget_root4_entry));

        // widget_root5: Tests the case of adding a widget region at root
        // level that is explicitly invisible and within layer bounds.
        let mut widget_root5_entry = StrongWidgetNodeEntry::new(
            Rc::new(RefCell::new(Box::new(EmptyPaintedTestWidget { id: 1 }))),
            WeakWidgetLayerEntry::new(),
            WeakRegionTreeEntry::new(),
            1,
        );
        let widget_root5_region_info = RegionInfo {
            size: Size::new(10.0, 8.0),
            internal_anchor: Anchor {
                h_align: HAlign::Left,
                v_align: VAlign::Top,
            },
            parent_anchor: Anchor {
                h_align: HAlign::Left,
                v_align: VAlign::Top,
            },
            parent_anchor_type: ParentAnchorType::Layer,
            anchor_offset: Point::new(80.0, 40.0),
        };
        let widget_root5_explicit_visibility = false;
        region_tree
            .add_widget_region(
                &mut widget_root5_entry,
                widget_root5_region_info.clone(),
                WidgetNodeType::Painted,
                widget_root5_explicit_visibility,
                &mut widgets_just_shown,
                &mut widgets_just_hidden,
            )
            .unwrap();
        let widget_root5_expected_rect = Rect::new(
            widget_root5_region_info.anchor_offset,
            widget_root5_region_info.size,
        );
        assert_region(
            &region_tree.roots[5].borrow().region,
            &Region::new_test_region(
                widget_root5_entry
                    .assigned_region()
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .region
                    .id,
                widget_root5_expected_rect,
                widget_root5_expected_rect.to_physical(scale_factor),
                widget_root5_region_info,
                None,
                layer_rect,
                widget_root5_explicit_visibility,
                layer_explicit_visibility,
                true,
            ),
        );
        // This region should not have been marked dirty since it is
        // explicitly invisible.
        assert!(!region_tree.dirty_widgets.contains(&widget_root5_entry));
        assert!(!widgets_just_shown.contains(&widget_root5_entry));

        // widget_root6: Tests the case of adding a widget region at root
        // level that is explicitly invisible and within layer bounds.
        let mut widget_root6_entry = StrongWidgetNodeEntry::new(
            Rc::new(RefCell::new(Box::new(EmptyPaintedTestWidget { id: 2 }))),
            WeakWidgetLayerEntry::new(),
            WeakRegionTreeEntry::new(),
            2,
        );
        let widget_root6_region_info = RegionInfo {
            size: Size::new(10.0, 8.0),
            internal_anchor: Anchor {
                h_align: HAlign::Left,
                v_align: VAlign::Top,
            },
            parent_anchor: Anchor {
                h_align: HAlign::Left,
                v_align: VAlign::Top,
            },
            parent_anchor_type: ParentAnchorType::Layer,
            anchor_offset: Point::new(300.0, 40.0),
        };
        let widget_root6_explicit_visibility = true;
        region_tree
            .add_widget_region(
                &mut widget_root6_entry,
                widget_root6_region_info.clone(),
                WidgetNodeType::Painted,
                widget_root6_explicit_visibility,
                &mut widgets_just_shown,
                &mut widgets_just_hidden,
            )
            .unwrap();
        let widget_root6_expected_rect = Rect::new(
            widget_root6_region_info.anchor_offset,
            widget_root6_region_info.size,
        );
        assert_region(
            &region_tree.roots[6].borrow().region,
            &Region::new_test_region(
                widget_root6_entry
                    .assigned_region()
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .region
                    .id,
                widget_root6_expected_rect,
                widget_root6_expected_rect.to_physical(scale_factor),
                widget_root6_region_info,
                None,
                layer_rect,
                widget_root6_explicit_visibility,
                layer_explicit_visibility,
                false,
            ),
        );
        // This region should not have been marked dirty since it is
        // outside the layer bounds.
        assert!(!region_tree.dirty_widgets.contains(&widget_root6_entry));
        assert!(!widgets_just_shown.contains(&widget_root6_entry));

        // widget_root0_0_0: Tests the case of adding a widget region that
        // is a child of a container region that is explicitly visible and
        // within layer bounds.
        let mut widget_root0_0_0_entry = StrongWidgetNodeEntry::new(
            Rc::new(RefCell::new(Box::new(EmptyPaintedTestWidget { id: 3 }))),
            WeakWidgetLayerEntry::new(),
            WeakRegionTreeEntry::new(),
            3,
        );
        let widget_root0_0_0_region_info = RegionInfo {
            size: Size::new(10.0, 8.0),
            internal_anchor: Anchor {
                h_align: HAlign::Left,
                v_align: VAlign::Top,
            },
            parent_anchor: Anchor {
                h_align: HAlign::Left,
                v_align: VAlign::Top,
            },
            parent_anchor_type: ParentAnchorType::ContainerRegion(container_root0_0_ref.clone()),
            anchor_offset: Point::new(2.0, 2.0),
        };
        let widget_root0_0_0_explicit_visibility = true;
        region_tree
            .add_widget_region(
                &mut widget_root0_0_0_entry,
                widget_root0_0_0_region_info.clone(),
                WidgetNodeType::Painted,
                widget_root0_0_0_explicit_visibility,
                &mut widgets_just_shown,
                &mut widgets_just_hidden,
            )
            .unwrap();
        let widget_root0_0_0_expected_rect = Rect::new(
            container_root0_0_expected_rect.pos() + widget_root0_0_0_region_info.anchor_offset,
            widget_root0_0_0_region_info.size,
        );
        assert_region(
            &region_tree.roots[0].borrow().children.as_ref().unwrap()[0]
                .borrow()
                .children
                .as_ref()
                .unwrap()[0]
                .borrow()
                .region,
            &Region::new_test_region(
                widget_root0_0_0_entry
                    .assigned_region()
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .region
                    .id,
                widget_root0_0_0_expected_rect,
                widget_root0_0_0_expected_rect.to_physical(scale_factor),
                widget_root0_0_0_region_info,
                None,
                container_root0_0_expected_rect,
                widget_root0_0_0_explicit_visibility,
                layer_explicit_visibility
                    && container_root0_explicit_visibility
                    && container_root0_0_explicit_visibility,
                true,
            ),
        );
        assert!(region_tree.dirty_widgets.contains(&widget_root0_0_0_entry));
        assert!(widgets_just_shown.contains(&widget_root0_0_0_entry));

        // widget_root1_0: Tests the case of adding a widget region that
        // is a child of a container region that is explicitly invisible
        // and within layer bounds.
        let mut widget_root1_0_entry = StrongWidgetNodeEntry::new(
            Rc::new(RefCell::new(Box::new(EmptyPaintedTestWidget { id: 4 }))),
            WeakWidgetLayerEntry::new(),
            WeakRegionTreeEntry::new(),
            4,
        );
        let widget_root1_0_region_info = RegionInfo {
            size: Size::new(10.0, 8.0),
            internal_anchor: Anchor {
                h_align: HAlign::Left,
                v_align: VAlign::Top,
            },
            parent_anchor: Anchor {
                h_align: HAlign::Left,
                v_align: VAlign::Top,
            },
            parent_anchor_type: ParentAnchorType::ContainerRegion(container_root1_ref.clone()),
            anchor_offset: Point::new(2.0, 2.0),
        };
        let widget_root1_0_explicit_visibility = true;
        region_tree
            .add_widget_region(
                &mut widget_root1_0_entry,
                widget_root1_0_region_info.clone(),
                WidgetNodeType::Painted,
                widget_root1_0_explicit_visibility,
                &mut widgets_just_shown,
                &mut widgets_just_hidden,
            )
            .unwrap();
        let widget_root1_0_expected_rect = Rect::new(
            container_root1_expected_rect.pos() + widget_root1_0_region_info.anchor_offset,
            widget_root1_0_region_info.size,
        );
        assert_region(
            &region_tree.roots[1].borrow().children.as_ref().unwrap()[0]
                .borrow()
                .region,
            &Region::new_test_region(
                widget_root1_0_entry
                    .assigned_region()
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .region
                    .id,
                widget_root1_0_expected_rect,
                widget_root1_0_expected_rect.to_physical(scale_factor),
                widget_root1_0_region_info,
                None,
                container_root1_expected_rect,
                widget_root1_0_explicit_visibility,
                layer_explicit_visibility && container_root1_explicit_visibility,
                true,
            ),
        );
        // This region should not have been marked dirty since its parent
        // region is explicitly invisible.
        assert!(!region_tree.dirty_widgets.contains(&widget_root1_0_entry));
        assert!(!widgets_just_shown.contains(&widget_root1_0_entry));

        // widget_root2_0: Tests the case of adding a widget region that
        // is a child of a container region that is explicitly visible
        // but not within layer bounds.
        let mut widget_root2_0_entry = StrongWidgetNodeEntry::new(
            Rc::new(RefCell::new(Box::new(EmptyPaintedTestWidget { id: 5 }))),
            WeakWidgetLayerEntry::new(),
            WeakRegionTreeEntry::new(),
            5,
        );
        let widget_root2_0_region_info = RegionInfo {
            size: Size::new(10.0, 8.0),
            internal_anchor: Anchor {
                h_align: HAlign::Left,
                v_align: VAlign::Top,
            },
            parent_anchor: Anchor {
                h_align: HAlign::Left,
                v_align: VAlign::Top,
            },
            parent_anchor_type: ParentAnchorType::ContainerRegion(container_root2_ref.clone()),
            anchor_offset: Point::new(2.0, 2.0),
        };
        let widget_root2_0_explicit_visibility = true;
        region_tree
            .add_widget_region(
                &mut widget_root2_0_entry,
                widget_root2_0_region_info.clone(),
                WidgetNodeType::Painted,
                widget_root2_0_explicit_visibility,
                &mut widgets_just_shown,
                &mut widgets_just_hidden,
            )
            .unwrap();
        let widget_root2_0_expected_rect = Rect::new(
            container_root2_expected_rect.pos() + widget_root2_0_region_info.anchor_offset,
            widget_root2_0_region_info.size,
        );
        assert_region(
            &region_tree.roots[2].borrow().children.as_ref().unwrap()[0]
                .borrow()
                .region,
            &Region::new_test_region(
                widget_root2_0_entry
                    .assigned_region()
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .region
                    .id,
                widget_root2_0_expected_rect,
                widget_root2_0_expected_rect.to_physical(scale_factor),
                widget_root2_0_region_info,
                None,
                container_root2_expected_rect,
                widget_root2_0_explicit_visibility,
                layer_explicit_visibility && container_root2_explicit_visibility,
                false,
            ),
        );
        // This region should not have been marked dirty since its parent
        // region is not within the layer bounds.
        assert!(!region_tree.dirty_widgets.contains(&widget_root2_0_entry));
        assert!(!widgets_just_shown.contains(&widget_root2_0_entry));

        // --------------------------------------------------------------------------------------------
        // --------------------------------------------------------------------------------------------

        // TODO: more tests
    }

    fn assert_region(region: &Region, expected_region: &Region) {
        assert_eq!(region.id, expected_region.id);
        if !region.rect.partial_eq_with_epsilon(expected_region.rect) {
            panic!(
                "region.rect: {:?}, expected_region.rect: {:?}",
                &region.rect, &expected_region.rect
            );
        }
        assert_eq!(region.internal_anchor, expected_region.internal_anchor);
        assert_eq!(region.parent_anchor, expected_region.parent_anchor);
        if !region
            .anchor_offset
            .partial_eq_with_epsilon(expected_region.anchor_offset)
        {
            panic!(
                "region.anchor_offset: {:?}, expected_region.anchor_offset: {:?}",
                &region.anchor_offset, &expected_region.anchor_offset
            );
        }
        assert_eq!(
            region.last_rendered_texture_rect,
            expected_region.last_rendered_texture_rect
        );
        if !region
            .parent_rect
            .partial_eq_with_epsilon(expected_region.parent_rect)
        {
            panic!(
                "region.parent_rect: {:?}, expected_region.parent_rect: {:?}",
                &region.parent_rect, &expected_region.parent_rect
            );
        }
        assert_eq!(
            region.explicit_visibility,
            expected_region.explicit_visibility
        );

        // Regions that are explicitly invisible don't do a check if they are
        // within the layer bounds.
        if region.explicit_visibility {
            if region.is_within_layer_rect != expected_region.is_within_layer_rect {
                panic!("region.is_within_layer_rect: {}, expected_region.is_within_layer_rect: {}, region.rect: {:?}", region.is_within_layer_rect, expected_region.is_within_layer_rect, &region.rect);
            }
        }
    }
}
