use crate::{IdGroup, Message, Root, Tree};

pub trait Application: Sized {
    type TextureIDs: IdGroup;
    type WidgetIDs: IdGroup;

    /// An enum of custom messages if you wish.
    type CustomMessage: std::fmt::Debug;

    /// Process messages.
    fn on_message(
        &mut self,
        message: Message<Self::CustomMessage>,
        root: &mut Root<Self::TextureIDs>,
    );

    /// Construct your current widgets.
    fn view(&self, tree: &mut Tree<Self::TextureIDs, Self::WidgetIDs>);
}
