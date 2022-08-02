use super::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RenderOrder {
    Particle, // Top
    Actor,
    Item,
    Corpse, // Last
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glyph {
    pub glyph: FontCharType,
    pub color: ColorPair,
    pub render_order: RenderOrder,
}

impl Glyph {
    pub fn new(glyph: FontCharType, color: ColorPair, render_order: RenderOrder) -> Self {
        Glyph { glyph, color, render_order }
    }
}