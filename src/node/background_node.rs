use std::any::Any;

use crate::VG;

use super::PaintRegionInfo;

pub trait BackgroundNode {
    #[allow(unused)]
    fn on_user_event(&mut self, event: Box<dyn Any>) -> bool {
        false
    }

    #[allow(unused)]
    fn paint(&mut self, vg: &mut VG, region: &PaintRegionInfo) {}
}
