use crate::{texture, Event, Message, Root};

pub trait Application: Sized {
    /// Your texture IDs
    type TextureIDs: texture::IdGroup;

    /// An enum of custom messages if you wish.
    type CustomMessage: std::fmt::Debug;

    /// Process messages.
    fn on_message(
        &mut self,
        message: Message<Self::CustomMessage>,
        gr: &mut Root<Self::TextureIDs>,
    );

    /// Process raw events manually if you wish.
    fn on_raw_event(
        &mut self,
        _event: Event,
        _canvas: &mut Root<Self::TextureIDs>,
    ) {
    }

    /// Process animations manually if you wish.
    fn on_frame(&mut self, _canvas: &mut Root<Self::TextureIDs>) {}
}
