use crate::node::StrongBackgroundNodeEntry;
use crate::renderer::BackgroundLayerRenderer;
use crate::size::{PhysicalPoint, PhysicalSize, Point, ScaleFactor, Size};

pub(crate) struct BackgroundLayer {
    pub id: u64,
    pub z_order: i32,
    pub renderer: Option<BackgroundLayerRenderer>,
    pub is_dirty: bool,
    pub physical_outer_position: PhysicalPoint,
    pub size: Size,
    pub physical_size: PhysicalSize,

    pub assigned_node: StrongBackgroundNodeEntry,

    outer_position: Point,
    explicit_visibility: bool,
    window_visibility: bool,
    scale_factor: ScaleFactor,
}

impl BackgroundLayer {
    pub fn new(
        id: u64,
        z_order: i32,
        size: Size,
        outer_position: Point,
        explicit_visibility: bool,
        window_visibility: bool,
        scale_factor: ScaleFactor,
        assigned_node: StrongBackgroundNodeEntry,
    ) -> Self {
        Self {
            id,
            z_order,
            renderer: Some(BackgroundLayerRenderer::new()),
            size,
            physical_size: size.to_physical(scale_factor),
            outer_position,
            physical_outer_position: outer_position.to_physical(scale_factor),
            explicit_visibility,
            window_visibility,
            scale_factor,
            is_dirty: true,
            assigned_node,
        }
    }

    pub fn set_outer_position(&mut self, position: Point, scale_factor: ScaleFactor) {
        self.outer_position = position;
        self.physical_outer_position = position.to_physical(scale_factor);
    }

    pub fn set_explicit_visibility(&mut self, explicit_visibility: bool) {
        if self.explicit_visibility != explicit_visibility {
            self.explicit_visibility = explicit_visibility;
            self.is_dirty = self.is_visible();
        }
    }

    pub fn set_window_visibility(&mut self, visible: bool) {
        if self.window_visibility != visible {
            self.window_visibility = visible;
            self.is_dirty = self.is_visible();
        }
    }

    pub fn set_size(&mut self, size: Size, scale_factor: ScaleFactor) {
        if self.size != size || self.scale_factor != scale_factor {
            self.size = size;
            self.scale_factor = scale_factor;

            self.is_dirty = self.is_visible();
        }
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = self.is_visible();
    }

    pub fn is_visible(&self) -> bool {
        self.explicit_visibility && self.window_visibility
    }
}
