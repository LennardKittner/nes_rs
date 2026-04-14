use crate::{
    rendering::{
        custom_tile_frame::CustomTileFrame,
        frame::{
            Frame, SCREEN_HEIGHT, SCREEN_HEIGHT_IN_TILES, SCREEN_WIDTH, SCREEN_WIDTH_IN_TILES,
        },
    },
    rom::Rom,
};

fn draw_block(tile_map: &mut [u8], x: usize, y: usize) {
    tile_map[x + SCREEN_WIDTH_IN_TILES * y] = 0xB4;
    tile_map[x + 1 + SCREEN_WIDTH_IN_TILES * y] = 0xB5;
    tile_map[x + SCREEN_WIDTH_IN_TILES * (y + 1)] = 0xB6;
    tile_map[x + 1 + SCREEN_WIDTH_IN_TILES * (y + 1)] = 0xB7;
}

fn draw_big_block(tile_map: &mut [u8], x: usize, y: usize) {
    draw_block(tile_map, x, y);
    draw_block(tile_map, x + 2, y);
    draw_block(tile_map, x, y + 2);
    draw_block(tile_map, x + 2, y + 2);
}

fn draw_bush(tile_map: &mut [u8], attribute_table: &mut [[u8; 4]], x: usize, y: usize) {
    let palette = [0x22, 0x29, 0x1A, 0x0F];

    attribute_table[(x / 2) + (SCREEN_WIDTH_IN_TILES / 2) * (y / 2)] = palette;
    attribute_table[((x + 2) / 2) + (SCREEN_WIDTH_IN_TILES / 2) * (y / 2)] = palette;
    attribute_table[((x + 4) / 2) + (SCREEN_WIDTH_IN_TILES / 2) * (y / 2)] = palette;

    tile_map[x + SCREEN_WIDTH_IN_TILES * (y + 1)] = 0x35;
    tile_map[x + 1 + SCREEN_WIDTH_IN_TILES * (y + 1)] = 0x25;
    tile_map[x + 2 + SCREEN_WIDTH_IN_TILES * (y + 1)] = 0x25;
    tile_map[x + 3 + SCREEN_WIDTH_IN_TILES * (y + 1)] = 0x25;
    tile_map[x + 4 + SCREEN_WIDTH_IN_TILES * (y + 1)] = 0x25;
    tile_map[x + 5 + SCREEN_WIDTH_IN_TILES * (y + 1)] = 0x38;

    tile_map[x + 1 + SCREEN_WIDTH_IN_TILES * y] = 0x36;
    tile_map[x + 2 + SCREEN_WIDTH_IN_TILES * y] = 0x37;
    tile_map[x + 3 + SCREEN_WIDTH_IN_TILES * y] = 0x36;
    tile_map[x + 4 + SCREEN_WIDTH_IN_TILES * y] = 0x37;
}

fn draw_cloud(tile_map: &mut [u8], attribute_table: &mut [[u8; 4]], x: usize, y: usize) {
    let palette = [0x22, 0x30, 0x21, 0x0F];

    attribute_table[(x / 2) + (SCREEN_WIDTH_IN_TILES / 2) * (y / 2)] = palette;
    attribute_table[((x + 2) / 2) + (SCREEN_WIDTH_IN_TILES / 2) * (y / 2)] = palette;
    attribute_table[(x / 2) + (SCREEN_WIDTH_IN_TILES / 2) * ((y + 2) / 2)] = palette;
    attribute_table[((x + 2) / 2) + (SCREEN_WIDTH_IN_TILES / 2) * ((y + 2) / 2)] = palette;

    tile_map[x + SCREEN_WIDTH_IN_TILES * (y + 2)] = 0x39;
    tile_map[x + 1 + SCREEN_WIDTH_IN_TILES * (y + 2)] = 0x3A;
    tile_map[x + 2 + SCREEN_WIDTH_IN_TILES * (y + 2)] = 0x3B;
    tile_map[x + 3 + SCREEN_WIDTH_IN_TILES * (y + 2)] = 0x3C;

    tile_map[x + SCREEN_WIDTH_IN_TILES * (y + 1)] = 0x35;
    tile_map[x + 1 + SCREEN_WIDTH_IN_TILES * (y + 1)] = 0x25;
    tile_map[x + 2 + SCREEN_WIDTH_IN_TILES * (y + 1)] = 0x25;
    tile_map[x + 3 + SCREEN_WIDTH_IN_TILES * (y + 1)] = 0x38;

    tile_map[x + 1 + SCREEN_WIDTH_IN_TILES * y] = 0x36;
    tile_map[x + 2 + SCREEN_WIDTH_IN_TILES * y] = 0x37;
}

fn draw_coin(tile_map: &mut [u8], attribute_table: &mut [[u8; 4]], x: usize, y: usize) {
    let palette = [0x22, 0x27, 0x17, 0x0F];
    attribute_table[x / 2 + (SCREEN_WIDTH_IN_TILES / 2) * (y / 2)] = palette;
    tile_map[x + SCREEN_WIDTH_IN_TILES * y] = 0x2E;
}

/// generates a frame containing my GitHub icon
pub fn generate_frame(rom: &Rom) -> Frame {
    let mut tile_map = vec![0x24u8; 960];
    let mut attribute_table =
        vec![[0x22, 0x36, 0x17, 0x0F]; SCREEN_WIDTH_IN_TILES / 2 * SCREEN_HEIGHT_IN_TILES / 2];

    for y in ((SCREEN_HEIGHT_IN_TILES - 4)..SCREEN_HEIGHT_IN_TILES).step_by(2) {
        for x in (0..SCREEN_WIDTH_IN_TILES).step_by(2) {
            draw_block(&mut tile_map, x, y);
        }
    }
    draw_big_block(&mut tile_map, 10, 18);
    draw_big_block(&mut tile_map, 14, 18);
    draw_big_block(&mut tile_map, 18, 18);

    draw_big_block(&mut tile_map, 6, 14);
    draw_big_block(&mut tile_map, 10, 14);
    draw_big_block(&mut tile_map, 18, 14);
    draw_big_block(&mut tile_map, 22, 14);

    draw_big_block(&mut tile_map, 6, 10);
    draw_big_block(&mut tile_map, 10, 10);
    draw_big_block(&mut tile_map, 14, 10);
    draw_big_block(&mut tile_map, 18, 10);
    draw_big_block(&mut tile_map, 22, 10);

    draw_big_block(&mut tile_map, 14, 6);

    draw_big_block(&mut tile_map, 10, 2);
    draw_big_block(&mut tile_map, 14, 2);
    draw_big_block(&mut tile_map, 18, 2);

    draw_bush(&mut tile_map, &mut attribute_table, 20, 8);

    draw_coin(&mut tile_map, &mut attribute_table, 8, 9);
    draw_coin(&mut tile_map, &mut attribute_table, 9, 9);
    draw_coin(&mut tile_map, &mut attribute_table, 10, 9);
    draw_coin(&mut tile_map, &mut attribute_table, 11, 9);

    draw_cloud(&mut tile_map, &mut attribute_table, 4, 5);

    let mut cf = CustomTileFrame::new(SCREEN_WIDTH, SCREEN_HEIGHT, tile_map, attribute_table);
    cf.update(rom, 1);

    cf.frame
}
