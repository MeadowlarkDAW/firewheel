use crate::{Anchor, HAlign, Point, Rect, Size, VAlign};
use fnv::{FnvHashMap, FnvHashSet};
use std::cell::{Ref, RefCell, RefMut};
use std::hash::Hash;
use std::rc::Rc;

pub(crate) struct RegionTree {
    next_id: u64,
    entries: FnvHashMap<RegionID, SharedRegionTreeEntry>,
    roots: Vec<SharedRegionTreeEntry>,
    pub dirty_regions: FnvHashSet<RegionID>,
    pub clear_rects: Vec<Rect>,
    layer_size: Size,
}

impl RegionTree {
    pub fn new(layer_size: Size) -> Self {
        Self {
            next_id: 0,
            entries: FnvHashMap::default(),
            roots: Vec::new(),
            dirty_regions: FnvHashSet::default(),
            clear_rects: Vec::new(),
            layer_size,
        }
    }

    pub fn new_region(
        &mut self,
        size: Size,
        internal_anchor: Anchor,
        parent_anchor: Anchor,
        parent_anchor_type: ParentAnchorType,
        anchor_offset: Point,
        is_invisible: bool,
    ) -> Result<RegionID, ()> {
        let new_id = RegionID(self.next_id);
        let mut new_entry = SharedRegionTreeEntry {
            shared: Rc::new(RefCell::new(RegionTreeEntry {
                region: Region {
                    id: new_id,
                    size,
                    internal_anchor,
                    parent_anchor,
                    parent_anchor_type,
                    anchor_offset,
                    rect: Rect::default(),        // This will be overwritten
                    parent_rect: Rect::default(), // This will be overwritten
                    last_rendered_rect: None,
                    is_invisible,
                },
                parent: None,
                children: Vec::new(),
            })),
        };

        let parent_rect = match parent_anchor_type {
            ParentAnchorType::Layer => {
                self.roots.push(new_entry.clone());

                Rect {
                    pos: Point { x: 0.0, y: 0.0 },
                    size: self.layer_size,
                }
            }
            ParentAnchorType::Region(id) => {
                let parent_rect = if let Some(parent_entry) = self.entries.get_mut(&id) {
                    let parent_rect = {
                        let mut parent_entry = parent_entry.borrow_mut();
                        parent_entry.children.push(new_entry.clone());

                        parent_entry.region.rect
                    };
                    {
                        let mut new_entry_ref = new_entry.borrow_mut();
                        new_entry_ref.parent = Some(parent_entry.clone());
                    }

                    parent_rect
                } else {
                    return Err(());
                };

                parent_rect
            }
        };

        {
            let mut new_entry_ref = new_entry.borrow_mut();
            new_entry_ref.region.update_parent_rect(parent_rect, true);
            new_entry_ref.mark_dirty(&mut self.dirty_regions, &mut self.clear_rects);
        }

        self.next_id += 1;
        Ok(new_id)
    }

    pub fn remove_region(&mut self, id: RegionID) -> Result<(), ()> {
        let mut entry = self.entries.remove(&id).ok_or_else(|| ())?;
        let mut entry_ref = entry.borrow_mut();

        if !entry_ref.children.is_empty() {
            return Err(());
        }

        if let Some(parent_entry) = &mut entry_ref.parent {
            let mut parent_entry = parent_entry.borrow_mut();
            let mut remove_i = None;
            for (i, entry) in parent_entry.children.iter().enumerate() {
                if entry.borrow().region.id == id {
                    remove_i = Some(i);
                    break;
                }
            }
            if let Some(i) = remove_i {
                parent_entry.children.remove(i);
            }
        } else {
            let mut remove_i = None;
            for (i, entry) in self.roots.iter().enumerate() {
                if entry.borrow().region.id == id {
                    remove_i = Some(i);
                    break;
                }
            }
            if let Some(i) = remove_i {
                self.roots.remove(i);
            }
        }

        if let Some(rect) = entry_ref.region.last_rendered_rect {
            self.clear_rects.push(rect);
        } // else this region was never rendered

        Ok(())
    }

    pub fn modify_region(
        &mut self,
        id: RegionID,
        new_size: Option<Size>,
        new_internal_anchor: Option<Anchor>,
        new_parent_anchor: Option<Anchor>,
        new_anchor_offset: Option<Point>,
    ) -> Result<(), ()> {
        let entry = self.entries.get_mut(&id).ok_or_else(|| ())?;
        entry.borrow_mut().modify(
            new_size,
            new_internal_anchor,
            new_parent_anchor,
            new_anchor_offset,
            &mut self.dirty_regions,
            &mut self.clear_rects,
        );
        Ok(())
    }

    pub fn mark_region_dirty(&mut self, id: RegionID) {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry
                .borrow_mut()
                .mark_dirty(&mut self.dirty_regions, &mut self.clear_rects);
        }
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
}

#[derive(Debug, Clone)]
struct SharedRegionTreeEntry {
    shared: Rc<RefCell<RegionTreeEntry>>,
}

impl SharedRegionTreeEntry {
    fn borrow(&self) -> Ref<'_, RegionTreeEntry> {
        RefCell::borrow(&self.shared)
    }

    fn borrow_mut(&mut self) -> RefMut<'_, RegionTreeEntry> {
        RefCell::borrow_mut(&self.shared)
    }
}

#[derive(Debug, Clone)]
struct RegionTreeEntry {
    region: Region,
    parent: Option<SharedRegionTreeEntry>,
    children: Vec<SharedRegionTreeEntry>,
}

impl RegionTreeEntry {
    fn modify(
        &mut self,
        new_size: Option<Size>,
        new_internal_anchor: Option<Anchor>,
        new_parent_anchor: Option<Anchor>,
        new_anchor_offset: Option<Point>,
        dirty_regions: &mut FnvHashSet<RegionID>,
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
            self.mark_dirty(dirty_regions, clear_rects);
            let new_rect = self.region.rect;

            for child_entry in self.children.iter_mut() {
                child_entry
                    .borrow_mut()
                    .update_parent_rect(new_rect, dirty_regions, clear_rects);
            }
        }
    }

    fn update_parent_rect(
        &mut self,
        parent_rect: Rect,
        dirty_regions: &mut FnvHashSet<RegionID>,
        clear_rects: &mut Vec<Rect>,
    ) {
        if self.region.update_parent_rect(parent_rect, false) {
            self.mark_dirty(dirty_regions, clear_rects);

            for child_entry in self.children.iter_mut() {
                child_entry.borrow_mut().update_parent_rect(
                    self.region.rect,
                    dirty_regions,
                    clear_rects,
                );
            }
        }
    }

    fn mark_dirty(
        &mut self,
        dirty_regions: &mut FnvHashSet<RegionID>,
        clear_rects: &mut Vec<Rect>,
    ) {
        if !self.region.is_invisible {
            if dirty_regions.insert(self.region.id) {
                if let Some(rect) = self.region.last_rendered_rect {
                    clear_rects.push(rect);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegionID(u64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Region {
    pub id: RegionID,
    pub size: Size,
    pub rect: Rect,
    pub internal_anchor: Anchor,
    pub parent_anchor: Anchor,
    pub parent_anchor_type: ParentAnchorType,
    pub anchor_offset: Point,
    pub last_rendered_rect: Option<Rect>,
    pub parent_rect: Rect,
    pub is_invisible: bool,
}

impl Region {
    fn update_rect(&mut self) {
        self.update_parent_rect(self.parent_rect, true);
    }

    fn update_parent_rect(&mut self, parent_rect: Rect, force_update: bool) -> bool {
        let mut changed = false;
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

        if changed || force_update {
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

        changed || force_update
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentAnchorType {
    Layer,
    Region(RegionID),
}
