use crate::rendering::frame::{Frame, SCREEN_WIDTH};

#[derive(Debug, Copy, Clone)]
pub struct SpriteColor {
    pub color: (u8, u8, u8),
    pub behind_background: bool,
    pub transparent: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct BackgroundColor {
    pub color: (u8, u8, u8),
    pub transparent: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct ScanlinePixel {
    pub background_color: BackgroundColor,
    pub sprite_color: SpriteColor,
    pub sprite_zero_opaque: bool,
}

impl ScanlinePixel {
    pub const fn new() -> ScanlinePixel {
        ScanlinePixel {
            background_color: BackgroundColor {
                color: (0, 0, 0),
                transparent: true,
            },
            sprite_color: SpriteColor {
                color: (0, 0, 0),
                behind_background: false,
                transparent: true,
            },
            sprite_zero_opaque: false,
        }
    }

    pub fn get_combined_color(&self) -> (u8, u8, u8) {
        match (
            self.background_color.transparent,
            self.sprite_color.behind_background,
            self.sprite_color.transparent,
        ) {
            (_, false, false) => self.sprite_color.color,
            (false, true, false) => self.background_color.color,
            (true, true, false) => self.sprite_color.color,
            (_, _, true) => self.background_color.color,
        }
    }
}

#[derive(Debug)]
pub struct Scanline {
    pub data: [ScanlinePixel; SCREEN_WIDTH],
}

impl Scanline {
    const EMPTY_SCANLINE: Scanline = Scanline::new();

    pub const fn new() -> Scanline {
        Scanline {
            data: [ScanlinePixel::new(); SCREEN_WIDTH],
        }
    }

    pub fn write_scanline(&self, frame: &mut Frame, y: usize) {
        for x in 0..256 {
            frame.set_pixel(x, y, self.data[x].get_combined_color())
        }
    }

    pub fn clear(&mut self) {
        *self = Self::EMPTY_SCANLINE;
    }
}
