use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    // 7  bit  0
    // ---- ----
    // VSO. ....
    // |||| ||||
    // |||+-++++- PPU open bus. Returns stale PPU bus contents.
    // ||+------- Sprite overflow. The intent was for this flag to be set
    // ||         whenever more than eight sprites appear on a scanline, but a
    // ||         hardware bug causes the actual behavior to be more complicated
    // ||         and generate false positives as well as false negatives; see
    // ||         PPU sprite evaluation. This flag is set during sprite
    // ||         evaluation and cleared at dot 1 (the second dot) of the
    // ||         pre-render line.
    // |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
    // |          a nonzero background pixel; cleared at dot 1 of the pre-render
    // |          line.  Used for raster timing.
    // +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
    //            Set at dot 1 of line 241 (the line *after* the post-render
    //            line); cleared after reading $2002 and at dot 1 of the
    //            pre-render line.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct StatusRegister: u8 {
        const UNUSED_0        = 0b0000_0001;
        const UNUSED_1        = 0b0000_0010;
        const UNUSED_2        = 0b0000_0100;
        const UNUSED_3        = 0b0000_1000;
        const UNUSED_4        = 0b0001_0000;
        const SPRITE_OVERFLOW = 0b0010_0000;
        const SPRITE_ZERO_HIT = 0b0100_0000;
        const VERTICAL_BLANK  = 0b1000_0000;
    }
}

impl StatusRegister {
    pub fn new() -> Self {
        Self::empty()
    }

    pub fn sprite_overflow(&self) -> bool {
        self.contains(Self::SPRITE_OVERFLOW)
    }

    pub fn set_sprite_overflow(&mut self, overflow: bool) {
        self.set(Self::SPRITE_OVERFLOW, overflow);
    }

    pub fn sprite_zero_hit(&self) -> bool {
        self.contains(Self::SPRITE_ZERO_HIT)
    }

    pub fn set_sprite_zero_hit(&mut self, hit: bool) {
        self.set(Self::SPRITE_ZERO_HIT, hit);
    }

    pub fn vertical_blank(&self) -> bool {
        self.contains(Self::VERTICAL_BLANK)
    }

    pub fn set_vertical_blank(&mut self, v_blank: bool) {
        self.set(Self::VERTICAL_BLANK, v_blank);
    }

    pub fn update(&mut self, data: u8) {
        self.0 = Self::from_bits(data).unwrap().0;
    }
}

impl Default for StatusRegister {
    fn default() -> Self {
        Self::new()
    }
}
