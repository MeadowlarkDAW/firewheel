use fnv::FnvHashSet;

use crate::node::StrongWidgetNodeEntry;

/// A set of widgets optimized for iteration.
pub(crate) struct WidgetNodeSet<A: Clone + Send + Sync + 'static> {
    unique_ids: FnvHashSet<u64>,
    entries: Vec<StrongWidgetNodeEntry<A>>,
}

impl<A: Clone + Send + Sync + 'static> WidgetNodeSet<A> {
    pub fn new() -> Self {
        Self {
            unique_ids: FnvHashSet::default(),
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, widget_entry: &StrongWidgetNodeEntry<A>) {
        if self.unique_ids.insert(widget_entry.unique_id()) {
            self.entries.push(widget_entry.clone());
        }
    }

    pub fn remove(&mut self, widget_entry: &StrongWidgetNodeEntry<A>) {
        if self.unique_ids.remove(&widget_entry.unique_id()) {
            let mut remove_i = None;
            for (i, entry) in self.entries.iter().enumerate() {
                if entry.unique_id() == widget_entry.unique_id() {
                    remove_i = Some(i);
                    break;
                }
            }
            if let Some(i) = remove_i {
                self.entries.remove(i);
            }
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut StrongWidgetNodeEntry<A>> {
        self.entries.iter_mut()
    }

    pub fn pop(&mut self) -> Option<StrongWidgetNodeEntry<A>> {
        self.entries.pop()
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
    pub fn contains(&self, widget_entry: &StrongWidgetNodeEntry<A>) -> bool {
        self.unique_ids.contains(&widget_entry.unique_id())
    }
}
