use super::allocator;

#[derive(Debug)]
pub enum Allocation {
    Partial {
        layer: usize,
        region: allocator::Region,
        hi_dpi: bool,
    },
    Full {
        layer: usize,
        hi_dpi: bool,
    },
}

impl Allocation {
    pub fn position(&self) -> (f32, f32) {
        match self {
            Allocation::Partial { region, .. } => region.position(),
            Allocation::Full { .. } => (0.0, 0.0),
        }
    }

    pub fn size(&self) -> (f32, f32) {
        match self {
            Allocation::Partial { region, .. } => region.size(),
            Allocation::Full { .. } => {
                (super::ATLAS_SIZE as f32, super::ATLAS_SIZE as f32)
            }
        }
    }

    pub fn layer(&self) -> usize {
        match self {
            Allocation::Partial { layer, .. } => *layer,
            Allocation::Full { layer, .. } => *layer,
        }
    }
}
