use crate::Point;

pub enum Primitive {}

pub struct Texture {
    pub texture_id_hash: u64,
    pub position: Point<u16>,
    pub rotation: f32,
}
