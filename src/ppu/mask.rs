use bitflags::bitflags;

bitflags! {
    // 7  bit  0
    // ---- ----
    // BGRs bMmG
    // |||| ||||
    // |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
    // |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
    // |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    // |||| +---- 1: Show background
    // |||+------ 1: Show sprites
    // ||+------- Emphasize red (green on PAL/Dendy)
    // |+-------- Emphasize green (red on PAL/Dendy)
    // +--------- Emphasize blue
    pub struct MaskRegister: u8 {
        const GREYSCALE       = 0b0000_0001;
        const BACKGROUND_LEFT = 0b0000_0010;
        const SPRITES_LEFT    = 0b0000_0100;
        const BACKGROUND      = 0b0000_1000;
        const SPRITES         = 0b0001_0000;
        const RED             = 0b0010_0000;
        const GREEN           = 0b0100_0000;
        const BLUE            = 0b1000_0000;
    }
}

impl MaskRegister {
    pub fn new() -> Self {
        Self::empty()
    }

    pub fn update(&mut self, data: u8) {
        self.0 = Self::from_bits(data).unwrap().0;
    }

    pub fn is_grayscale(&self) -> bool {
        self.contains(Self::GREYSCALE)
    }

    pub fn show_sprites_left(&self) -> bool {
        self.contains(Self::SPRITES_LEFT)
    }

    pub fn show_background(&self) -> bool {
        self.contains(Self::BACKGROUND)
    }

    pub fn show_background_left(&self) -> bool {
        self.contains(Self::BACKGROUND_LEFT)
    }

    pub fn show_sprites(&self) -> bool {
        self.contains(Self::SPRITES)
    }

    pub fn get_emphasis_index(&self) -> u8 {
        self.bits() >> 5
    }
}

impl Default for MaskRegister {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn emphasis() {
        let mut r = MaskRegister::new();
        assert_eq!(r.get_emphasis_index(), 0);
        r.insert(MaskRegister::RED);
        assert_eq!(r.get_emphasis_index(), 1);
        r.remove(MaskRegister::RED);
        r.insert(MaskRegister::GREEN);
        assert_eq!(r.get_emphasis_index(), 2);
        r.remove(MaskRegister::GREEN);
        r.insert(MaskRegister::BLUE);
        assert_eq!(r.get_emphasis_index(), 4);
        r.insert(MaskRegister::RED);
        assert_eq!(r.get_emphasis_index(), 5);
    }
}
