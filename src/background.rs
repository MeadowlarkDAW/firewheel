use crate::{Color, IdGroup, Point};

pub enum Background<ID: IdGroup> {
    SolidColor(Color),
    Texture(ID),
    MultipleTextures(Vec<(ID, Point<u16>)>),
}

impl<ID: IdGroup> Background<ID> {
    pub(crate) fn hash_to_u64(&self) -> Background<u64> {
        match self {
            Background::SolidColor(color) => Background::SolidColor(*color),
            Background::Texture(id) => Background::Texture(id.hash_to_u64()),
            Background::MultipleTextures(textures) => {
                Background::MultipleTextures(
                    textures
                        .iter()
                        .map(|(id, position)| -> (u64, Point<u16>) {
                            (id.hash_to_u64(), *position)
                        })
                        .collect(),
                )
            }
        }
    }
}
