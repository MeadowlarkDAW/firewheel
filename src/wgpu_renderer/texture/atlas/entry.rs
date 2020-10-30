use super::allocation;

#[derive(Debug)]
pub enum Entry {
    Contiguous(allocation::Allocation),
    Fragmented {
        size: (f32, f32),
        fragments: Vec<Fragment>,
    },
}

impl Entry {
    pub fn size(&self) -> (f32, f32) {
        match self {
            Entry::Contiguous(allocation) => allocation.size(),
            Entry::Fragmented { size, .. } => *size,
        }
    }
}

#[derive(Debug)]
pub struct Fragment {
    pub position: (u32, u32),
    pub allocation: allocation::Allocation,
}
