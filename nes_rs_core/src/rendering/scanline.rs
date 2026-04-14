use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::rendering::frame::{Frame, SCREEN_WIDTH};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SpriteColor {
    pub color: (u8, u8, u8),
    pub behind_background: bool,
    pub transparent: bool,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BackgroundColor {
    pub color: (u8, u8, u8),
    pub transparent: bool,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ScanlinePixel {
    pub background_color: BackgroundColor,
    pub sprite_color: SpriteColor,
    pub sprite_zero_opaque: bool,
}

impl Default for ScanlinePixel {
    fn default() -> Self {
        Self::new()
    }
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

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Scanline {
    #[serde_as(as = "[_; SCREEN_WIDTH]")]
    pub data: [ScanlinePixel; SCREEN_WIDTH],
}

impl Default for Scanline {
    fn default() -> Self {
        Self::new()
    }
}

impl Scanline {
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
        for pixel in &mut self.data {
            pixel.sprite_color = SpriteColor {
                color: (0, 0, 0),
                behind_background: false,
                transparent: true,
            };
            pixel.sprite_zero_opaque = false;
        }
    }
}
