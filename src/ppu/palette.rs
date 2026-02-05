// generate palettes https://bisqwit.iki.fi/utils/nespalette.php

pub const PALETTE_SIZE_E: usize = 64;
const PALETTE_SIZE_B: usize = PALETTE_SIZE_E * 3;
const NUMBER_PALETTES: usize = 8;
#[allow(dead_code)]
const SYSTEM_PALETTE_SIZE_B: usize = PALETTE_SIZE_B * NUMBER_PALETTES;
#[allow(dead_code)]
const SYSTEM_PALETTE_SIZE_E: usize = PALETTE_SIZE_E * NUMBER_PALETTES;

pub type Pallet = [(u8, u8, u8); PALETTE_SIZE_E];

// bgr
// 000 -> normal
// 001 -> red   = 1
// 010 -> green = 2
// 100 -> blue  = 4
// 011 -> green & blue = 3
// 101 -> blue & red   = 5
// 110 -> blue & green = 6
// 111 -> all          = 7
#[derive(Clone)]
pub struct SystemPalette {
    palettes: [Pallet; NUMBER_PALETTES],
}

impl SystemPalette {
    fn zero() -> Self {
        SystemPalette {
            palettes: [[(0, 0, 0); PALETTE_SIZE_E]; 8],
        }
    }

    pub fn new() -> Self {
        Self::from_raw(include_bytes!("../../system.palette")).unwrap()
    }

    pub fn from_single_palette(palette: Pallet) -> Self {
        SystemPalette {
            palettes: [palette; NUMBER_PALETTES],
        }
    }

    pub fn from_raw(buffer: &[u8]) -> Option<Self> {
        if !buffer.len().is_multiple_of(PALETTE_SIZE_B) {
            return None;
        }
        let mut system_palette = SystemPalette::zero();
        for palette_index in 0..(buffer.len() / (PALETTE_SIZE_B)) {
            ((palette_index * PALETTE_SIZE_B)..(palette_index * PALETTE_SIZE_B + PALETTE_SIZE_B))
                .step_by(3)
                .map(|i| (buffer[i], buffer[i + 1], buffer[i + 2]))
                .enumerate()
                .for_each(|(i, rgb)| {
                    system_palette.palettes[palette_index][i] = rgb;
                });
        }

        Some(system_palette)
    }

    pub fn get_palette(&self, idx: usize) -> Pallet {
        self.palettes[idx]
    }

    pub fn get_color(&self, palette_idx: usize, color_idx: usize) -> (u8, u8, u8) {
        self.palettes[palette_idx][color_idx]
    }
}

impl Default for SystemPalette {
    fn default() -> Self {
        SystemPalette::new()
    }
}
