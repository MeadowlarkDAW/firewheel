use crate::canvas::StrongWidgetEntry;
use crate::event::{InputEvent, PointerEvent};
use crate::{
    Anchor, EventCapturedStatus, HAlign, Point, Rect, Size, VAlign, WidgetRegionType,
    WidgetRequests,
};
use fnv::{FnvHashMap, FnvHashSet};
use std::cell::{Ref, RefCell, RefMut};
use std::hash::Hash;
use std::rc::{Rc, Weak};

// TODO: Write unit tests for this monstrosity.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegionInfo {
    pub size: Size,
    pub internal_anchor: Anchor,
    pub parent_anchor: Anchor,
    pub parent_anchor_type: ParentAnchorType,
    pub anchor_offset: Point,
}

pub(crate) struct RegionTree<MSG> {
    next_region_id: u64,
    roots: Vec<StrongRegionTreeEntry<MSG>>,
    widget_id_to_assigned_region: FnvHashMap<u64, StrongRegionTreeEntry<MSG>>,
    container_id_to_assigned_region: FnvHashMap<u64, StrongRegionTreeEntry<MSG>>,
    dirty_regions: FnvHashSet<u64>,
    clear_rects: Vec<Rect>,
    widgets_just_shown: FnvHashSet<u64>,
    widgets_just_hidden: FnvHashSet<u64>,
    layer_rect: Rect,
}

impl<MSG> RegionTree<MSG> {
    pub fn new(layer_size: Size) -> Self {
        Self {
            next_region_id: 0,
            roots: Vec::new(),
            widget_id_to_assigned_region: FnvHashMap::default(),
            container_id_to_assigned_region: FnvHashMap::default(),
            dirty_regions: FnvHashSet::default(),
            clear_rects: Vec::new(),
            widgets_just_shown: FnvHashSet::default(),
            widgets_just_hidden: FnvHashSet::default(),
            layer_rect: Rect::new(Point::new(0.0, 0.0), layer_size),
        }
    }

    pub fn new_container_region(
        &mut self,
        size: Size,
        internal_anchor: Anchor,
        parent_anchor: Anchor,
        parent_anchor_type: ParentAnchorType,
        anchor_offset: Point,
        explicit_visibility: bool,
    ) -> Result<ContainerRegionID, ()> {
        let new_id = ContainerRegionID(self.next_region_id);
        self.next_region_id += 1;
        let mut new_entry = StrongRegionTreeEntry {
            shared: Rc::new(RefCell::new(RegionTreeEntry {
                region: Region {
                    id: new_id.0,
                    size,
                    internal_anchor,
                    parent_anchor,
                    parent_anchor_type,
                    anchor_offset,
                    rect: Rect::default(),        // This will be overwritten
                    parent_rect: Rect::default(), // This will be overwritten
                    last_rendered_rect: None,
                    is_container: true,
                    explicit_visibility,
                    is_within_layer_rect: false,
                },
                parent: None,
                children: Some(Vec::new()),
                assigned_widget: None,
            })),
            region_id: new_id.0,
        };

        let parent_rect = match parent_anchor_type {
            ParentAnchorType::Layer => {
                self.roots.push(new_entry.clone());

                self.layer_rect
            }
            ParentAnchorType::ContainerRegion(id) => {
                let parent_rect = if let Some(parent_entry) =
                    self.container_id_to_assigned_region.get_mut(&id.0)
                {
                    let parent_rect = {
                        let mut parent_entry_ref = parent_entry.borrow_mut();
                        if let Some(children) = &mut parent_entry_ref.children {
                            children.push(new_entry.clone());
                        } else {
                            panic!("Parent region is not a container region");
                        }
                        parent_entry_ref.region.rect
                    };
                    {
                        new_entry.borrow_mut().parent = Some(parent_entry.downgrade());
                    }

                    parent_rect
                } else {
                    return Err(());
                };

                parent_rect
            }
        };
        {
            let mut new_entry = new_entry.borrow_mut();

            new_entry.update_parent_rect(
                parent_rect,
                &mut self.dirty_regions,
                &mut self.clear_rects,
            );

            new_entry.check_if_within_layer_bounds(
                self.layer_rect,
                &mut self.dirty_regions,
                &mut self.clear_rects,
                &mut self.widgets_just_shown,
                &mut self.widgets_just_hidden,
            );
        }

        self.container_id_to_assigned_region
            .insert(new_id.0, new_entry);

        Ok(new_id)
    }

    pub fn remove_container_region(&mut self, id: ContainerRegionID) -> Result<(), ()> {
        let mut entry = self
            .container_id_to_assigned_region
            .remove(&id.0)
            .ok_or_else(|| ())?;
        let mut entry_ref = entry.borrow_mut();

        if let Some(children) = &entry_ref.children {
            if !children.is_empty() {
                return Err(());
            }
        } else {
            panic!("region was not a container region");
        }

        // Remove this child entry from its parent.
        if let Some(parent_entry) = entry_ref.parent.take() {
            let parent_entry = parent_entry.upgrade().unwrap();
            let mut parent_entry = parent_entry.borrow_mut();

            if let Some(children) = &mut parent_entry.children {
                let mut remove_i = None;
                for (i, e) in children.iter().enumerate() {
                    if e.region_id == id.0 {
                        remove_i = Some(i);
                        break;
                    }
                }
                if let Some(i) = remove_i {
                    children.remove(i);
                }
            } else {
                panic!("parent region was not a container region");
            }
        } else {
            // This entry had no parent, so remove it from the root entries instead.
            let mut remove_i = None;
            for (i, e) in self.roots.iter().enumerate() {
                if e.region_id == id.0 {
                    remove_i = Some(i);
                    break;
                }
            }
            if let Some(i) = remove_i {
                self.roots.remove(i);
            }
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
    ) -> Result<(), ()> {
        let mut entry_ref = self
            .container_id_to_assigned_region
            .get_mut(&id.0)
            .ok_or_else(|| ())?
            .borrow_mut();

        if entry_ref.children.is_none() {
            panic!("region was not a container region");
        }

        entry_ref.modify(
            new_size,
            new_internal_anchor,
            new_parent_anchor,
            new_anchor_offset,
            &mut self.dirty_regions,
            &mut self.clear_rects,
            self.layer_rect,
            &mut self.widgets_just_shown,
            &mut self.widgets_just_hidden,
        );

        Ok(())
    }

    pub fn mark_container_region_dirty(&mut self, id: ContainerRegionID) -> Result<(), ()> {
        let mut entry_ref = self
            .container_id_to_assigned_region
            .get_mut(&id.0)
            .ok_or_else(|| ())?
            .borrow_mut();

        if entry_ref.children.is_none() {
            panic!("region was not a container region");
        }

        Ok(entry_ref.mark_dirty(&mut self.dirty_regions, &mut self.clear_rects, None, true))
    }

    pub fn set_container_region_explicit_visibility(
        &mut self,
        id: ContainerRegionID,
        explicit_visibility: bool,
    ) -> Result<(), ()> {
        let mut entry_ref = self
            .container_id_to_assigned_region
            .get_mut(&id.0)
            .ok_or_else(|| ())?
            .borrow_mut();

        if entry_ref.children.is_none() {
            panic!("region was not a container region");
        }

        Ok(entry_ref.set_explicit_visibilty(
            explicit_visibility,
            &mut self.dirty_regions,
            &mut self.clear_rects,
            self.layer_rect,
            &mut self.widgets_just_shown,
            &mut self.widgets_just_hidden,
        ))
    }

    pub fn add_widget_region(
        &mut self,
        assigned_widget: &mut StrongWidgetEntry<MSG>,
        region_info: RegionInfo,
        //listens_to_pointer_events: bool,
        region_type: WidgetRegionType,
        explicit_visibility: bool,
    ) -> Result<(), ()> {
        if self
            .widget_id_to_assigned_region
            .contains_key(&assigned_widget.unique_id())
        {
            return Err(());
        }

        let new_id = ContainerRegionID(self.next_region_id);
        self.next_region_id += 1;

        let mut entry = StrongRegionTreeEntry {
            shared: Rc::new(RefCell::new(RegionTreeEntry {
                region: Region {
                    id: new_id.0,
                    size: region_info.size,
                    internal_anchor: region_info.internal_anchor,
                    parent_anchor: region_info.parent_anchor,
                    parent_anchor_type: region_info.parent_anchor_type,
                    anchor_offset: region_info.anchor_offset,
                    rect: Rect::default(),        // This will be overwritten
                    parent_rect: Rect::default(), // This will be overwritten
                    last_rendered_rect: None,
                    is_container: true,
                    explicit_visibility,
                    is_within_layer_rect: false,
                },
                parent: None,
                children: None,
                assigned_widget: Some(RegionAssignedWidget {
                    widget: assigned_widget.clone(),
                    listens_to_pointer_events: false,
                    region_type,
                }),
            })),
            region_id: new_id.0,
        };

        assigned_widget.set_assigned_region(entry.downgrade());

        let parent_rect = match region_info.parent_anchor_type {
            ParentAnchorType::Layer => {
                self.roots.push(entry.clone());

                self.layer_rect
            }
            ParentAnchorType::ContainerRegion(id) => {
                let parent_rect = if let Some(parent_entry) =
                    self.container_id_to_assigned_region.get_mut(&id.0)
                {
                    let parent_rect = {
                        let mut parent_entry_ref = parent_entry.borrow_mut();
                        if let Some(children) = &mut parent_entry_ref.children {
                            children.push(entry.clone());
                        } else {
                            panic!("Parent region is not a container region");
                        }
                        parent_entry_ref.region.rect
                    };
                    {
                        entry.borrow_mut().parent = Some(parent_entry.downgrade());
                    }

                    parent_rect
                } else {
                    return Err(());
                };

                parent_rect
            }
        };

        self.widget_id_to_assigned_region
            .insert(assigned_widget.unique_id(), entry.clone());

        {
            let weak_entry = entry.downgrade();
            let mut entry_ref = entry.borrow_mut();

            entry_ref
                .assigned_widget
                .as_mut()
                .unwrap()
                .widget
                .set_assigned_region(weak_entry);

            entry_ref.update_parent_rect(
                parent_rect,
                &mut self.dirty_regions,
                &mut self.clear_rects,
            );

            entry_ref.check_if_within_layer_bounds(
                self.layer_rect,
                &mut self.dirty_regions,
                &mut self.clear_rects,
                &mut self.widgets_just_shown,
                &mut self.widgets_just_hidden,
            );
        }

        Ok(())
    }

    pub fn remove_widget_region(&mut self, widget: &StrongWidgetEntry<MSG>) {
        let mut entry = if let Some(entry) = self
            .widget_id_to_assigned_region
            .remove(&widget.unique_id())
        {
            entry
        } else {
            return;
        };

        let entry_region_id = entry.region_id;
        let mut entry_ref = entry.borrow_mut();

        if entry_ref.children.is_some() {
            panic!("region was not a widget region");
        }

        self.dirty_regions.remove(&entry_ref.region.id);
        if let Some(rect) = entry_ref.region.last_rendered_rect.take() {
            self.clear_rects.push(rect);
        }

        self.widgets_just_shown.remove(&widget.unique_id());
        self.widgets_just_hidden.remove(&widget.unique_id());

        // Remove this child entry from its parent.
        if let Some(parent_entry) = entry_ref.parent.take() {
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
            }
        }
    }

    pub fn modify_widget_region(
        &mut self,
        widget: &StrongWidgetEntry<MSG>,
        new_size: Option<Size>,
        new_internal_anchor: Option<Anchor>,
        new_parent_anchor: Option<Anchor>,
        new_anchor_offset: Option<Point>,
    ) -> Result<(), ()> {
        widget
            .assigned_region()
            .upgrade()
            .ok_or_else(|| ())?
            .borrow_mut()
            .modify(
                new_size,
                new_internal_anchor,
                new_parent_anchor,
                new_anchor_offset,
                &mut self.dirty_regions,
                &mut self.clear_rects,
                self.layer_rect,
                &mut self.widgets_just_shown,
                &mut self.widgets_just_hidden,
            );

        Ok(())
    }

    pub fn mark_widget_region_dirty(&mut self, widget: &StrongWidgetEntry<MSG>) -> Result<(), ()> {
        widget
            .assigned_region()
            .upgrade()
            .ok_or_else(|| ())?
            .borrow_mut()
            .mark_dirty(&mut self.dirty_regions, &mut self.clear_rects, None, true);

        Ok(())
    }

    pub fn set_widget_region_explicit_visibility(
        &mut self,
        widget: &StrongWidgetEntry<MSG>,
        explicit_visibility: bool,
    ) -> Result<(), ()> {
        widget
            .assigned_region()
            .upgrade()
            .ok_or_else(|| ())?
            .borrow_mut()
            .set_explicit_visibilty(
                explicit_visibility,
                &mut self.dirty_regions,
                &mut self.clear_rects,
                self.layer_rect,
                &mut self.widgets_just_shown,
                &mut self.widgets_just_hidden,
            );

        Ok(())
    }

    pub fn set_widget_region_listens_to_pointer_events(
        &mut self,
        widget: &StrongWidgetEntry<MSG>,
        listens: bool,
    ) -> Result<(), ()> {
        widget
            .assigned_region()
            .upgrade()
            .ok_or_else(|| ())?
            .borrow_mut()
            .assigned_widget
            .as_mut()
            .unwrap()
            .listens_to_pointer_events = listens;

        Ok(())
    }

    pub fn set_layer_size(&mut self, size: Size) {
        if self.layer_rect.size() == size {
            return;
        }
        self.layer_rect.set_size(size);

        for entry in self.roots.iter_mut() {
            let mut entry = entry.borrow_mut();

            entry.update_parent_rect(
                self.layer_rect,
                &mut self.dirty_regions,
                &mut self.clear_rects,
            );

            entry.check_if_within_layer_bounds(
                self.layer_rect,
                &mut self.dirty_regions,
                &mut self.clear_rects,
                &mut self.widgets_just_shown,
                &mut self.widgets_just_hidden,
            );
        }
    }

    pub fn layer_just_shown(&mut self) {
        for entry in self.roots.iter_mut() {
            let mut entry = entry.borrow_mut();

            entry.check_if_within_layer_bounds(
                self.layer_rect,
                &mut self.dirty_regions,
                &mut self.clear_rects,
                &mut self.widgets_just_shown,
                &mut self.widgets_just_hidden,
            );
            entry.mark_dirty(
                &mut self.dirty_regions,
                &mut self.clear_rects,
                Some(&mut self.widgets_just_shown),
                true,
            );
        }
    }

    pub fn layer_just_hidden(&mut self) {
        for entry in self.roots.iter_mut() {
            entry.borrow_mut().set_just_hidden(
                &mut self.dirty_regions,
                &mut self.clear_rects,
                &mut self.widgets_just_hidden,
                true,
            );
        }
    }

    pub fn is_dirty(&self) -> bool {
        !self.dirty_regions.is_empty() || !self.clear_rects.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }

    pub fn handle_pointer_event(
        &mut self,
        event: PointerEvent,
        msg_out_queue: &mut Vec<MSG>,
    ) -> Option<(StrongWidgetEntry<MSG>, WidgetRequests)> {
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
    fn borrow(&self) -> Ref<'_, RegionTreeEntry<MSG>> {
        RefCell::borrow(&self.shared)
    }

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

    fn upgrade(&self) -> Option<Rc<RefCell<RegionTreeEntry<MSG>>>> {
        self.shared.upgrade()
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
        widget: StrongWidgetEntry<MSG>,
        requests: WidgetRequests,
    },
    InRegionButNotCaptured,
    NotInRegion,
}

struct RegionAssignedWidget<MSG> {
    widget: StrongWidgetEntry<MSG>,
    listens_to_pointer_events: bool,
    region_type: WidgetRegionType,
}

pub(super) struct RegionTreeEntry<MSG> {
    region: Region,
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

    fn set_explicit_visibilty(
        &mut self,
        explicit_visibility: bool,
        dirty_regions: &mut FnvHashSet<u64>,
        clear_rects: &mut Vec<Rect>,
        layer_rect: Rect,
        widgets_just_shown: &mut FnvHashSet<u64>,
        widgets_just_hidden: &mut FnvHashSet<u64>,
    ) {
        if self.region.explicit_visibility != explicit_visibility {
            self.region.explicit_visibility = explicit_visibility;

            if explicit_visibility {
                let layer_visibility_changed = self.check_if_within_layer_bounds(
                    layer_rect,
                    dirty_regions,
                    clear_rects,
                    widgets_just_shown,
                    widgets_just_hidden,
                );
                if !layer_visibility_changed && self.region.is_visible() {
                    self.mark_dirty(dirty_regions, clear_rects, None, false);
                }
            } else {
                self.set_just_hidden(dirty_regions, clear_rects, widgets_just_hidden, true);
            }
        }
    }

    fn modify(
        &mut self,
        new_size: Option<Size>,
        new_internal_anchor: Option<Anchor>,
        new_parent_anchor: Option<Anchor>,
        new_anchor_offset: Option<Point>,
        dirty_regions: &mut FnvHashSet<u64>,
        clear_rects: &mut Vec<Rect>,
        layer_rect: Rect,
        widgets_just_shown: &mut FnvHashSet<u64>,
        widgets_just_hidden: &mut FnvHashSet<u64>,
    ) {
        let mut changed = false;
        if let Some(new_size) = new_size {
            if self.region.size != new_size {
                self.region.size = new_size;
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

        if changed {
            self.region.update_rect();

            let layer_visibility_changed = self.check_if_within_layer_bounds(
                layer_rect,
                dirty_regions,
                clear_rects,
                widgets_just_shown,
                widgets_just_hidden,
            );
            if !layer_visibility_changed && self.region.is_visible() {
                self.mark_dirty(dirty_regions, clear_rects, None, false);
            }

            if let Some(children) = &mut self.children {
                for child_entry in children.iter_mut() {
                    child_entry.borrow_mut().update_parent_rect(
                        self.region.rect,
                        dirty_regions,
                        clear_rects,
                    );
                }
            }
        }
    }

    fn check_if_within_layer_bounds(
        &mut self,
        layer_rect: Rect,
        dirty_regions: &mut FnvHashSet<u64>,
        clear_rects: &mut Vec<Rect>,
        widgets_just_shown: &mut FnvHashSet<u64>,
        widgets_just_hidden: &mut FnvHashSet<u64>,
    ) -> bool {
        let mut changed = false;

        if self.region.explicit_visibility {
            let is_within_layer_rect = layer_rect.overlaps_with_rect(self.region.rect);

            if self.region.is_within_layer_rect {
                if !is_within_layer_rect {
                    // The region is no longer within the layer rect.
                    self.region.is_within_layer_rect = false;
                    changed = true;

                    self.set_just_hidden(dirty_regions, clear_rects, widgets_just_hidden, false);
                }
            } else {
                if is_within_layer_rect {
                    // The region is now within the layer rect.
                    self.region.is_within_layer_rect = true;
                    changed = true;

                    self.mark_dirty(dirty_regions, clear_rects, Some(widgets_just_shown), false);
                }
            }
        }

        if let Some(children) = &mut self.children {
            for child_entry in children.iter_mut() {
                child_entry.borrow_mut().check_if_within_layer_bounds(
                    layer_rect,
                    dirty_regions,
                    clear_rects,
                    widgets_just_shown,
                    widgets_just_hidden,
                );
            }
        }

        changed
    }

    fn update_parent_rect(
        &mut self,
        parent_rect: Rect,
        dirty_regions: &mut FnvHashSet<u64>,
        clear_rects: &mut Vec<Rect>,
    ) {
        if self.region.update_parent_rect(parent_rect, false) {
            self.mark_dirty(dirty_regions, clear_rects, None, false);

            if let Some(children) = &mut self.children {
                for child_entry in children.iter_mut() {
                    child_entry.borrow_mut().update_parent_rect(
                        self.region.rect,
                        dirty_regions,
                        clear_rects,
                    );
                }
            }
        }
    }

    fn mark_dirty(
        &mut self,
        dirty_regions: &mut FnvHashSet<u64>,
        clear_rects: &mut Vec<Rect>,
        widgets_just_shown: Option<&mut FnvHashSet<u64>>,
        recurse: bool,
    ) {
        if self.region.is_visible() {
            if let Some(children) = &mut self.children {
                if recurse {
                    // Get around the borrow checker.
                    if let Some(widgets_just_shown) = widgets_just_shown {
                        for child_entry in children.iter_mut() {
                            child_entry.borrow_mut().mark_dirty(
                                dirty_regions,
                                clear_rects,
                                Some(widgets_just_shown),
                                true,
                            );
                        }
                    } else {
                        for child_entry in children.iter_mut() {
                            child_entry.borrow_mut().mark_dirty(
                                dirty_regions,
                                clear_rects,
                                None,
                                true,
                            );
                        }
                    }
                }
            } else {
                let assigned_widget_info = self.assigned_widget.as_ref().unwrap();

                if let Some(widgets_just_shown) = widgets_just_shown {
                    widgets_just_shown.insert(assigned_widget_info.widget.unique_id());
                }

                if let WidgetRegionType::Painted = assigned_widget_info.region_type {
                    dirty_regions.insert(self.region.id);
                    if let Some(rect) = self.region.last_rendered_rect.take() {
                        clear_rects.push(rect);
                    }
                }
            }
        }
    }

    fn set_just_hidden(
        &mut self,
        dirty_regions: &mut FnvHashSet<u64>,
        clear_rects: &mut Vec<Rect>,
        widgets_just_hidden: &mut FnvHashSet<u64>,
        recurse: bool,
    ) {
        if self.region.explicit_visibility {
            if let Some(children) = &mut self.children {
                if recurse {
                    for child_entry in children.iter_mut() {
                        child_entry.borrow_mut().set_just_hidden(
                            dirty_regions,
                            clear_rects,
                            widgets_just_hidden,
                            true,
                        );
                    }
                }
            } else {
                dirty_regions.remove(&self.region.id);
                if let Some(rect) = self.region.last_rendered_rect.take() {
                    clear_rects.push(rect);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContainerRegionID(u64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Region {
    pub id: u64,
    pub rect: Rect,
    pub size: Size,
    pub internal_anchor: Anchor,
    pub parent_anchor: Anchor,
    pub parent_anchor_type: ParentAnchorType,
    pub anchor_offset: Point,
    pub last_rendered_rect: Option<Rect>,
    pub parent_rect: Rect,
    pub is_container: bool,
    pub explicit_visibility: bool,
    pub is_within_layer_rect: bool,
}

impl Region {
    fn update_rect(&mut self) {
        self.update_parent_rect(self.parent_rect, true);
    }

    fn update_parent_rect(&mut self, parent_rect: Rect, force_update: bool) -> bool {
        let mut changed = force_update;
        let parent_anchor_pos_x = match self.parent_anchor.h_align {
            HAlign::Left => {
                if self.parent_rect.x() != parent_rect.x() {
                    changed = true;
                }
                parent_rect.x()
            }
            HAlign::Center => {
                if self.parent_rect.center_x() != parent_rect.center_x() {
                    changed = true;
                }
                parent_rect.center_x()
            }
            HAlign::Right => {
                if self.parent_rect.x2() != parent_rect.x2() {
                    changed = true;
                }
                parent_rect.x2()
            }
        };
        let parent_anchor_pos_y = match self.parent_anchor.v_align {
            VAlign::Top => {
                if self.parent_rect.y() != parent_rect.y() {
                    changed = true;
                }
                parent_rect.y()
            }
            VAlign::Center => {
                if self.parent_rect.center_y() != parent_rect.center_y() {
                    changed = true;
                }
                parent_rect.center_y()
            }
            VAlign::Bottom => {
                if self.parent_rect.y2() != parent_rect.y2() {
                    changed = true;
                }
                parent_rect.y2()
            }
        };

        self.parent_rect = parent_rect;

        if changed {
            let internal_anchor_pos_x = parent_anchor_pos_x + self.anchor_offset.x;
            let internal_anchor_pos_y = parent_anchor_pos_y + self.anchor_offset.y;

            let new_x = match self.internal_anchor.h_align {
                HAlign::Left => internal_anchor_pos_x,
                HAlign::Center => internal_anchor_pos_x - (self.size.width() / 2.0),
                HAlign::Right => internal_anchor_pos_x - self.size.width(),
            };
            let new_y = match self.internal_anchor.v_align {
                VAlign::Top => internal_anchor_pos_y,
                VAlign::Center => internal_anchor_pos_y - (self.size.height() / 2.0),
                VAlign::Bottom => internal_anchor_pos_y - self.size.height(),
            };

            self.rect.set_pos(Point::new(new_x, new_y));
        }

        changed
    }

    #[inline]
    pub fn is_visible(&self) -> bool {
        self.explicit_visibility && self.is_within_layer_rect
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentAnchorType {
    Layer,
    ContainerRegion(ContainerRegionID),
}
