use super::Widget;
use crate::IdGroup;
use std::collections::HashMap;

pub struct WidgetTree<TexID: IdGroup, WidgetID: IdGroup> {
    widgets: HashMap<WidgetID, Box<dyn Widget<TextureIDs = TexID>>>,
}

impl<TexID: IdGroup, WidgetID: IdGroup> WidgetTree<TexID, WidgetID> {
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
        }
    }
}
