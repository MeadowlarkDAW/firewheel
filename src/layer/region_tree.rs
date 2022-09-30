use crate::canvas::{StrongWidgetEntry, WidgetRef};
use crate::event::{Event, MouseEvent};
use crate::{Anchor, EventCapturedStatus, HAlign, Point, Rect, Size, VAlign, WidgetRequests};
use fnv::{FnvHashMap, FnvHashSet};
use std::cell::{Ref, RefCell, RefMut};
use std::hash::Hash;
use std::rc::{Rc, Weak};

pub(crate) struct RegionTree<MSG> {
    next_region_id: u64,
    entries: FnvHashMap<u64, StrongRegionTreeEntry<MSG>>,
    roots: Vec<StrongRegionTreeEntry<MSG>>,
    widget_id_to_assigned_region: FnvHashMap<u64, StrongRegionTreeEntry<MSG>>,
    dirty_regions: FnvHashSet<u64>,
    clear_rects: Vec<Rect>,
    widgets_just_shown: FnvHashSet<u64>,
    widgets_just_hidden: FnvHashSet<u64>,
    layer_size: Size,
}

impl<MSG> RegionTree<MSG> {
    pub fn new(layer_size: Size) -> Self {
        Self {
            next_region_id: 0,
            entries: FnvHashMap::default(),
            roots: Vec::new(),
            widget_id_to_assigned_region: FnvHashMap::default(),
            dirty_regions: FnvHashSet::default(),
            clear_rects: Vec::new(),
            widgets_just_shown: FnvHashSet::default(),
            widgets_just_hidden: FnvHashSet::default(),
            layer_size,
        }
    }

    pub fn new_container_region(
        &mut self,
        size: Size,
        internal_anchor: Anchor,
        parent_anchor: Anchor,
        parent_anchor_type: ParentAnchorType,
        anchor_offset: Point,
        visible: bool,
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
                    visible,
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

                Rect {
                    pos: Point { x: 0.0, y: 0.0 },
                    size: self.layer_size,
                }
            }
            ParentAnchorType::ContainerRegion(id) => {
                let parent_rect = if let Some(parent_entry) = self.entries.get_mut(&id.0) {
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
            new_entry.borrow_mut().update_parent_rect(
                parent_rect,
                &mut self.dirty_regions,
                &mut self.clear_rects,
            )
        }

        Ok(new_id)
    }

    pub fn remove_container_region(&mut self, id: ContainerRegionID) -> Result<(), ()> {
        let mut entry = self.entries.remove(&id.0).ok_or_else(|| ())?;
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
        let mut entry_ref = self.entries.get_mut(&id.0).ok_or_else(|| ())?.borrow_mut();

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
        );

        Ok(())
    }

    pub fn mark_container_region_dirty(&mut self, id: ContainerRegionID) -> Result<(), ()> {
        let mut entry_ref = self.entries.get_mut(&id.0).ok_or_else(|| ())?.borrow_mut();

        if entry_ref.children.is_none() {
            panic!("region was not a container region");
        }

        Ok(entry_ref.mark_dirty(&mut self.dirty_regions, &mut self.clear_rects, None))
    }

    pub fn layer_just_shown(&mut self) {
        for entry in self.roots.iter_mut() {
            entry.borrow_mut().mark_dirty(
                &mut self.dirty_regions,
                &mut self.clear_rects,
                Some(&mut self.widgets_just_shown),
            );
        }
    }

    pub fn layer_just_hidden(&mut self) {
        for entry in self.roots.iter_mut() {
            entry.borrow_mut().set_just_hidden(
                &mut self.dirty_regions,
                &mut self.clear_rects,
                &mut self.widgets_just_hidden,
            );
        }
    }

    pub fn set_container_region_visibility(
        &mut self,
        id: ContainerRegionID,
        visible: bool,
    ) -> Result<(), ()> {
        let mut entry_ref = self.entries.get_mut(&id.0).ok_or_else(|| ())?.borrow_mut();

        if entry_ref.children.is_none() {
            panic!("region was not a container region");
        }

        Ok(entry_ref.set_visibilty(
            visible,
            &mut self.dirty_regions,
            &mut self.clear_rects,
            &mut self.widgets_just_shown,
            &mut self.widgets_just_hidden,
        ))
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
    ) -> Result<(), ()> {
        let mut is_new_insert = false;
        let entry = self
            .widget_id_to_assigned_region
            .entry(assigned_widget.unique_id())
            .or_insert_with(|| {
                is_new_insert = true;

                let new_id = ContainerRegionID(self.next_region_id);
                self.next_region_id += 1;

                StrongRegionTreeEntry {
                    shared: Rc::new(RefCell::new(RegionTreeEntry {
                        region: Region {
                            id: new_id.0,
                            size,
                            internal_anchor,
                            parent_anchor,
                            parent_anchor_type,
                            anchor_offset,
                            rect: Rect::default(), // This will be overwritten
                            parent_rect: Rect::default(), // This will be overwritten
                            last_rendered_rect: None,
                            is_container: true,
                            visible,
                        },
                        parent: None,
                        children: None,
                        assigned_widget: Some(RegionAssignedWidget {
                            widget: assigned_widget,
                            listens_to_mouse_events,
                        }),
                    })),
                    region_id: new_id.0,
                }
            });

        let mut entry_ref = entry.shared.borrow_mut();
        let mut add_to_new_parent = is_new_insert;
        if !is_new_insert {
            entry_ref
                .assigned_widget
                .as_mut()
                .unwrap()
                .listens_to_mouse_events = listens_to_mouse_events;

            if entry_ref.region.parent_anchor_type != parent_anchor_type {
                add_to_new_parent = true;
                entry_ref.region.parent_anchor_type = parent_anchor_type;

                // Remove this child entry from its old parent.
                if let Some(old_parent_entry) = entry_ref.parent.take() {
                    let old_parent_entry = old_parent_entry.upgrade().unwrap();
                    let mut old_parent_entry = old_parent_entry.borrow_mut();

                    if let Some(children) = &mut old_parent_entry.children {
                        let mut remove_i = None;
                        for (i, e) in children.iter().enumerate() {
                            if e.region_id == entry.region_id {
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
                        if e.region_id == entry.region_id {
                            remove_i = Some(i);
                            break;
                        }
                    }
                    if let Some(i) = remove_i {
                        self.roots.remove(i);
                    }
                }
            }

            entry_ref.set_visibilty(
                visible,
                &mut self.dirty_regions,
                &mut self.clear_rects,
                &mut self.widgets_just_shown,
                &mut self.widgets_just_hidden,
            );

            entry_ref.modify(
                Some(size),
                Some(internal_anchor),
                Some(parent_anchor),
                Some(anchor_offset),
                &mut self.dirty_regions,
                &mut self.clear_rects,
            );
        }

        if add_to_new_parent {
            let parent_rect = match parent_anchor_type {
                ParentAnchorType::Layer => {
                    self.roots.push(entry.clone());

                    Rect {
                        pos: Point { x: 0.0, y: 0.0 },
                        size: self.layer_size,
                    }
                }
                ParentAnchorType::ContainerRegion(id) => {
                    let parent_rect = if let Some(parent_entry) = self.entries.get_mut(&id.0) {
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
                            entry_ref.parent = Some(parent_entry.downgrade());
                        }

                        parent_rect
                    } else {
                        return Err(());
                    };

                    parent_rect
                }
            };

            entry_ref.update_parent_rect(
                parent_rect,
                &mut self.dirty_regions,
                &mut self.clear_rects,
            )
        }

        Ok(())
    }

    pub fn remove_widget_region(&mut self, widget: &WidgetRef<MSG>) -> Result<(), ()> {
        let mut entry = self
            .widget_id_to_assigned_region
            .remove(&widget.unique_id())
            .ok_or_else(|| ())?;
        let entry_region_id = entry.region_id;
        let mut entry_ref = entry.borrow_mut();

        if entry_ref.children.is_some() {
            panic!("region was not a widget region");
        }

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

        Ok(())
    }

    pub fn mark_widget_region_dirty(&mut self, widget: &WidgetRef<MSG>) -> Result<(), ()> {
        let mut entry_ref = self
            .widget_id_to_assigned_region
            .get_mut(&widget.unique_id())
            .ok_or_else(|| ())?
            .borrow_mut();

        if entry_ref.children.is_some() {
            panic!("region was not a widget region");
        }

        Ok(entry_ref.mark_dirty(&mut self.dirty_regions, &mut self.clear_rects, None))
    }

    pub fn set_widget_region_visibility(
        &mut self,
        widget: &WidgetRef<MSG>,
        visible: bool,
    ) -> Result<(), ()> {
        let mut entry_ref = self
            .widget_id_to_assigned_region
            .get_mut(&widget.unique_id())
            .ok_or_else(|| ())?
            .borrow_mut();

        if entry_ref.children.is_some() {
            panic!("region was not a widget region");
        }

        Ok(entry_ref.set_visibilty(
            visible,
            &mut self.dirty_regions,
            &mut self.clear_rects,
            &mut self.widgets_just_shown,
            &mut self.widgets_just_hidden,
        ))
    }

    pub fn set_layer_size(&mut self, size: Size) {
        if self.layer_size == size {
            return;
        }
        self.layer_size = size;

        let new_rect = Rect {
            pos: Point { x: 0.0, y: 0.0 },
            size,
        };

        for entry in self.roots.iter_mut() {
            entry.borrow_mut().update_parent_rect(
                new_rect,
                &mut self.dirty_regions,
                &mut self.clear_rects,
            );
        }
    }

    pub fn is_dirty(&self) -> bool {
        !self.dirty_regions.is_empty() || !self.clear_rects.is_empty()
    }

    pub fn handle_mouse_event(
        &mut self,
        event: MouseEvent,
    ) -> Option<(StrongWidgetEntry<MSG>, WidgetRequests<MSG>)> {
        for region in self.roots.iter_mut() {
            match region.borrow_mut().handle_mouse_event(event) {
                MouseCapturedStatus::Captured { widget, requests } => {
                    return Some((widget, requests));
                }
                MouseCapturedStatus::InRegionButNotCaptured => {
                    return None;
                }
                MouseCapturedStatus::NotInRegion => {}
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

enum MouseCapturedStatus<MSG> {
    Captured {
        widget: StrongWidgetEntry<MSG>,
        requests: WidgetRequests<MSG>,
    },
    InRegionButNotCaptured,
    NotInRegion,
}

struct RegionAssignedWidget<MSG> {
    widget: StrongWidgetEntry<MSG>,
    listens_to_mouse_events: bool,
}

pub(super) struct RegionTreeEntry<MSG> {
    region: Region,
    parent: Option<WeakRegionTreeEntry<MSG>>,
    children: Option<Vec<StrongRegionTreeEntry<MSG>>>,
    assigned_widget: Option<RegionAssignedWidget<MSG>>,
}

impl<MSG> RegionTreeEntry<MSG> {
    fn handle_mouse_event(&mut self, mut event: MouseEvent) -> MouseCapturedStatus<MSG> {
        if self.region.visible {
            if self.region.rect.contains_point(event.position) {
                if let Some(assigned_widget) = &mut self.assigned_widget {
                    if assigned_widget.listens_to_mouse_events {
                        // Remove the region's offset from the position of the mouse event.
                        let temp_position = event.position;
                        let temp_prev_position = event.previous_position;
                        event.position -= self.region.rect.pos;
                        event.previous_position -= self.region.rect.pos;

                        let status = {
                            assigned_widget
                                .widget
                                .borrow_mut()
                                .on_event(&Event::Mouse(event))
                        };
                        let status = if let EventCapturedStatus::Captured(requests) = status {
                            MouseCapturedStatus::Captured {
                                widget: assigned_widget.widget.clone(),
                                requests,
                            }
                        } else {
                            MouseCapturedStatus::InRegionButNotCaptured
                        };

                        event.position = temp_position;
                        event.previous_position = temp_prev_position;

                        return status;
                    }
                } else if let Some(children) = &mut self.children {
                    for child_region in children.iter_mut() {
                        match child_region.borrow_mut().handle_mouse_event(event) {
                            MouseCapturedStatus::Captured { widget, requests } => {
                                return MouseCapturedStatus::Captured { widget, requests };
                            }
                            MouseCapturedStatus::InRegionButNotCaptured => {
                                return MouseCapturedStatus::InRegionButNotCaptured;
                            }
                            MouseCapturedStatus::NotInRegion => {}
                        }
                    }
                }

                return MouseCapturedStatus::InRegionButNotCaptured;
            }
        }

        MouseCapturedStatus::NotInRegion
    }

    fn set_visibilty(
        &mut self,
        visible: bool,
        dirty_regions: &mut FnvHashSet<u64>,
        clear_rects: &mut Vec<Rect>,
        widgets_just_shown: &mut FnvHashSet<u64>,
        widgets_just_hidden: &mut FnvHashSet<u64>,
    ) {
        if self.region.visible != visible {
            self.region.visible = visible;

            if visible {
                self.mark_dirty(dirty_regions, clear_rects, Some(widgets_just_shown));
            } else {
                self.set_just_hidden(dirty_regions, clear_rects, widgets_just_hidden);
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
            self.mark_dirty(dirty_regions, clear_rects, None);
            let new_rect = self.region.rect;

            if let Some(children) = &mut self.children {
                for child_entry in children.iter_mut() {
                    child_entry.borrow_mut().update_parent_rect(
                        new_rect,
                        dirty_regions,
                        clear_rects,
                    );
                }
            }
        }
    }

    fn update_parent_rect(
        &mut self,
        parent_rect: Rect,
        dirty_regions: &mut FnvHashSet<u64>,
        clear_rects: &mut Vec<Rect>,
    ) {
        if self.region.update_parent_rect(parent_rect, false) {
            self.mark_dirty(dirty_regions, clear_rects, None);

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
    ) {
        if self.region.visible {
            if let Some(children) = &mut self.children {
                // Get around the borrow checker.
                if let Some(widgets_just_shown) = widgets_just_shown {
                    for child_entry in children.iter_mut() {
                        child_entry.borrow_mut().mark_dirty(
                            dirty_regions,
                            clear_rects,
                            Some(widgets_just_shown),
                        );
                    }
                } else {
                    for child_entry in children.iter_mut() {
                        child_entry
                            .borrow_mut()
                            .mark_dirty(dirty_regions, clear_rects, None);
                    }
                }
            } else if let Some(widgets_just_shown) = widgets_just_shown {
                widgets_just_shown
                    .insert(self.assigned_widget.as_ref().unwrap().widget.unique_id());
            }

            if dirty_regions.insert(self.region.id) {
                if let Some(rect) = self.region.last_rendered_rect {
                    clear_rects.push(rect);
                }
            }
        }
    }

    fn set_just_hidden(
        &mut self,
        dirty_regions: &mut FnvHashSet<u64>,
        clear_rects: &mut Vec<Rect>,
        widgets_just_hidden: &mut FnvHashSet<u64>,
    ) {
        if self.region.visible {
            if let Some(children) = &mut self.children {
                for child_entry in children.iter_mut() {
                    child_entry.borrow_mut().set_just_hidden(
                        dirty_regions,
                        clear_rects,
                        widgets_just_hidden,
                    );
                }
            } else {
                dirty_regions.remove(&self.region.id);
                if let Some(rect) = self.region.last_rendered_rect {
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
    pub size: Size,
    pub rect: Rect,
    pub internal_anchor: Anchor,
    pub parent_anchor: Anchor,
    pub parent_anchor_type: ParentAnchorType,
    pub anchor_offset: Point,
    pub last_rendered_rect: Option<Rect>,
    pub parent_rect: Rect,
    pub is_container: bool,
    pub visible: bool,
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

            self.rect.pos.x = match self.internal_anchor.h_align {
                HAlign::Left => internal_anchor_pos_x,
                HAlign::Center => internal_anchor_pos_x - (self.size.width() / 2.0),
                HAlign::Right => internal_anchor_pos_x - self.size.width(),
            };
            self.rect.pos.y = match self.internal_anchor.v_align {
                VAlign::Top => internal_anchor_pos_y,
                VAlign::Center => internal_anchor_pos_y - (self.size.height() / 2.0),
                VAlign::Bottom => internal_anchor_pos_y - self.size.height(),
            };
        }

        changed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentAnchorType {
    Layer,
    ContainerRegion(ContainerRegionID),
}
