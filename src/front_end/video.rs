use nes_rs::{
    ppu::{palette::SystemPalette, PPU},
    rendering::{
        fps_frame::FPSFrame,
        frame::{Frame, SCREEN_HEIGHT, SCREEN_WIDTH, TILE_WIDTH},
        render_nametable, render_oam_table, render_oam_with_pos, write_tile,
    },
    ring_buffer::RingBuffer,
    rom::Rom,
};
use sdl2::{
    pixels::{Color, PixelFormatEnum},
    rect::{Point, Rect},
    render::{BlendMode, Canvas, ScaleMode, Texture, TextureCreator},
    video::{Window, WindowBuildError, WindowContext},
    VideoSubsystem,
};
use std::collections::HashMap;

use crate::{
    front_end::{
        FrontEndState, RewindBuffer, BACKGROUND_COLOR, GRID_PIXEL_IN_NES_PIXEL, HISTORY_SIZE,
        PALETTE_VIEWER_DIMENSIONS, SPRITE_TABLE_DIMENSIONS, SPRITE_VIEW_DIMENSIONS,
    },
    FONT_LETTERS_OFFSET, FONT_NUMBERS_OFFSET,
};

pub fn create_window(
    video_subsystem: &VideoSubsystem,
    title: &str,
    width: u32,
    height: u32,
) -> Result<Window, WindowBuildError> {
    video_subsystem
        .window(title, width, height)
        .resizable()
        .build()
}

pub fn creates_canvas_and_texture_creator(
    window: Window,
    logical_width: u32,
    logical_height: u32,
    integer_scaling: bool,
) -> (Canvas<Window>, TextureCreator<WindowContext>) {
    let mut canvas = window.into_canvas().build().unwrap();
    canvas
        .set_logical_size(logical_width, logical_height)
        .unwrap();
    if integer_scaling {
        canvas.set_integer_scale(true).unwrap();
    }
    let creator = canvas.texture_creator();
    (canvas, creator)
}

pub struct TextureCreators {
    pub main_creator: TextureCreator<WindowContext>,
    pub tile_map_creator: TextureCreator<WindowContext>,
    pub tile_creator: TextureCreator<WindowContext>,
    pub palette_creator: TextureCreator<WindowContext>,
    pub sprite_creator: TextureCreator<WindowContext>,
}

pub struct Textures<'a> {
    main_texture: Texture<'a>,
    rewind_texture: Texture<'a>,
    fps_texture: Texture<'a>,
    tile_texture: Texture<'a>,
    palette_texture: Texture<'a>,
    sprite_texture: Texture<'a>,
    nametable_textures: Vec<Texture<'a>>,
    nametable_grid_texture: Texture<'a>,
    sprite_table_grid_texture: Texture<'a>,
    sprite_view_grid_texture: Texture<'a>,
    tiles_grid_texture: Texture<'a>,
    main_screen_grid_texture: Texture<'a>,

    frame_buffer: Frame,
    fps_frame: FPSFrame,
    frame_counter: u32,
    system_palette: SystemPalette,
}

pub fn create_grid_texture<'a>(
    canvas: &mut Canvas<Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
    grid_width: u32,
    grid_height: u32,
    gap: u32,
    half_way_separator: bool,
) -> Texture<'a> {
    let mut texture = texture_creator
        .create_texture_target(PixelFormatEnum::RGBA8888, grid_width, grid_height)
        .unwrap();
    let gap = gap as i32;

    canvas
        .with_texture_canvas(&mut texture, |texture_canvas| {
            texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
            texture_canvas.clear();

            texture_canvas.set_draw_color(Color::GREEN);
            for i in 0..(grid_width as i32 / gap) {
                texture_canvas
                    .draw_line(
                        Point::new(i * gap, 0),
                        Point::new(i * gap, grid_height as i32 - 1),
                    )
                    .unwrap();
            }
            for i in 0..(grid_height as i32 / gap) {
                texture_canvas
                    .draw_line(
                        Point::new(0, i * gap),
                        Point::new(grid_width as i32 - 1, i * gap),
                    )
                    .unwrap();
            }
            if half_way_separator {
                texture_canvas.set_draw_color(Color::RED);
                texture_canvas
                    .draw_line(
                        Point::new(0, grid_height as i32 / 2),
                        Point::new(grid_width as i32 - 1, grid_height as i32 / 2),
                    )
                    .unwrap();

                texture_canvas
                    .draw_line(
                        Point::new(grid_width as i32 / 2, 0),
                        Point::new(grid_width as i32 / 2, grid_height as i32 - 1),
                    )
                    .unwrap();
            }
        })
        .unwrap();

    texture.set_blend_mode(BlendMode::Blend);
    texture.set_scale_mode(ScaleMode::Nearest);
    texture
}

impl<'a> Textures<'a> {
    pub fn new(
        front_end_state: &mut FrontEndState,
        texture_creators: &'a TextureCreators,
        system_palette: SystemPalette,
    ) -> Textures<'a> {
        let main_texture = texture_creators
            .main_creator
            .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
            .unwrap();

        let rewind_texture = texture_creators
            .main_creator
            .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
            .unwrap();

        let fps_texture = texture_creators
            .main_creator
            .create_texture_target(PixelFormatEnum::RGB24, 48, 8)
            .unwrap();

        let tile_texture = texture_creators
            .tile_creator
            .create_texture_target(
                PixelFormatEnum::RGB24,
                SCREEN_WIDTH as u32,
                SCREEN_HEIGHT as u32,
            )
            .unwrap();

        let palette_texture = texture_creators
            .palette_creator
            .create_texture_target(
                PixelFormatEnum::RGB24,
                PALETTE_VIEWER_DIMENSIONS.0,
                PALETTE_VIEWER_DIMENSIONS.1,
            )
            .unwrap();

        let sprite_texture = texture_creators
            .sprite_creator
            .create_texture_target(
                PixelFormatEnum::RGB24,
                SPRITE_VIEW_DIMENSIONS.0,
                SPRITE_VIEW_DIMENSIONS.1,
            )
            .unwrap();

        let mut nametable_textures = Vec::new();
        for _ in 0..4 {
            nametable_textures.push(
                texture_creators
                    .tile_map_creator
                    .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
                    .unwrap(),
            );
        }

        let nametable_grid_texture = create_grid_texture(
            &mut front_end_state.tile_map_canvas,
            &texture_creators.tile_map_creator,
            SCREEN_WIDTH as u32 * 2 * GRID_PIXEL_IN_NES_PIXEL,
            SCREEN_HEIGHT as u32 * 2 * GRID_PIXEL_IN_NES_PIXEL,
            TILE_WIDTH as u32 * GRID_PIXEL_IN_NES_PIXEL,
            true,
        );

        let sprite_table_grid_texture = create_grid_texture(
            &mut front_end_state.sprite_canvas,
            &texture_creators.sprite_creator,
            SPRITE_TABLE_DIMENSIONS.0 * GRID_PIXEL_IN_NES_PIXEL,
            SPRITE_TABLE_DIMENSIONS.1 * GRID_PIXEL_IN_NES_PIXEL,
            TILE_WIDTH as u32 * GRID_PIXEL_IN_NES_PIXEL,
            false,
        );

        let sprite_view_grid_texture = create_grid_texture(
            &mut front_end_state.sprite_canvas,
            &texture_creators.sprite_creator,
            SCREEN_WIDTH as u32 * GRID_PIXEL_IN_NES_PIXEL,
            SCREEN_HEIGHT as u32 * GRID_PIXEL_IN_NES_PIXEL,
            TILE_WIDTH as u32 * GRID_PIXEL_IN_NES_PIXEL,
            false,
        );

        let tiles_grid_texture = create_grid_texture(
            &mut front_end_state.tile_canvas,
            &texture_creators.tile_creator,
            SCREEN_WIDTH as u32 * GRID_PIXEL_IN_NES_PIXEL,
            SCREEN_HEIGHT as u32 * GRID_PIXEL_IN_NES_PIXEL,
            TILE_WIDTH as u32 * GRID_PIXEL_IN_NES_PIXEL,
            false,
        );

        let main_screen_grid_texture = create_grid_texture(
            &mut front_end_state.main_canvas,
            &texture_creators.main_creator,
            SCREEN_WIDTH as u32 * GRID_PIXEL_IN_NES_PIXEL,
            SCREEN_HEIGHT as u32 * GRID_PIXEL_IN_NES_PIXEL,
            TILE_WIDTH as u32 * GRID_PIXEL_IN_NES_PIXEL,
            false,
        );

        Textures {
            main_texture,
            rewind_texture,
            fps_texture,
            tile_texture,
            palette_texture,
            sprite_texture,
            nametable_textures,
            nametable_grid_texture,
            sprite_table_grid_texture,
            sprite_view_grid_texture,
            tiles_grid_texture,
            main_screen_grid_texture,
            frame_buffer: Frame::default(),
            fps_frame: FPSFrame::new(
                FONT_NUMBERS_OFFSET,
                FONT_LETTERS_OFFSET,
                [0x30, 0x30, 0x30, 0x30],
            ),
            system_palette,
            frame_counter: 0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_textures(
        &mut self,
        emulation_frame: &Frame,
        fps: u32,
        front_end_state: &mut FrontEndState,
        ppu: &PPU,
        rom: &Rom,
        font_chr_rom: &[u8],
        system_palette: &SystemPalette,
        rewind_buffer: &RewindBuffer,
    ) {
        self.main_texture
            .update(None, &emulation_frame.data, emulation_frame.width * 3)
            .unwrap();
        front_end_state
            .main_canvas
            .copy(&self.main_texture, None, None)
            .unwrap();

        if front_end_state.actions.show_fps {
            self.fps_frame.update(
                font_chr_rom,
                fps as usize,
                ppu.get_universal_background_color_idx(),
            );
            self.fps_texture
                .update(
                    None,
                    &self.fps_frame.frame.data,
                    self.fps_frame.frame.width * 3,
                )
                .unwrap();
            front_end_state
                .main_canvas
                .copy(
                    &self.fps_texture,
                    None,
                    Some(sdl2::rect::Rect::new(5, 5, 48, 8)),
                )
                .unwrap();
        }

        if front_end_state.actions.rewind_mode {
            self.update_rewind_ui(front_end_state, rewind_buffer);
        }

        if front_end_state.actions.show_grid {
            front_end_state
                .main_canvas
                .copy(&self.main_screen_grid_texture, None, None)
                .unwrap();
        }

        front_end_state.main_canvas.present();

        if front_end_state.actions.show_tile_map {
            self.nametable_textures
                .iter_mut()
                .enumerate()
                .for_each(|(i, texture)| {
                    render_nametable(ppu, rom, i, &mut self.frame_buffer, &self.system_palette);
                    texture
                        .update(None, &self.frame_buffer.data, self.frame_buffer.width * 3)
                        .unwrap()
                });

            self.nametable_textures
                .iter()
                .enumerate()
                .for_each(|(i, texture)| {
                    let i = i as i32;
                    let x: i32 = i % 2 * SCREEN_WIDTH as i32;
                    let y: i32 = if i < 2 { 0 } else { SCREEN_HEIGHT as i32 };
                    front_end_state
                        .tile_map_canvas
                        .copy(
                            texture,
                            None,
                            Some(sdl2::rect::Rect::new(
                                x * 2,
                                y * 2,
                                SCREEN_WIDTH as u32 * 2,
                                SCREEN_HEIGHT as u32 * 2,
                            )),
                        )
                        .unwrap()
                });

            if front_end_state.actions.show_grid {
                front_end_state
                    .tile_map_canvas
                    .copy(&self.nametable_grid_texture, None, None)
                    .unwrap();
            }

            front_end_state.tile_map_canvas.present();
        }

        if front_end_state.actions.show_palette {
            let palettes = (0..8).map(|idx| {
                let idx = idx * 4;
                [
                    ppu.read_palette_table(idx),
                    ppu.read_palette_table(idx + 1),
                    ppu.read_palette_table(idx + 2),
                    ppu.read_palette_table(idx + 3),
                ]
            });
            for (palette_idx, palette) in palettes.enumerate() {
                for (color_idx, &palette_entry) in palette.iter().enumerate() {
                    write_tile(
                        &mut self.frame_buffer,
                        color_idx * 8,
                        palette_idx * 8,
                        &[0u8; 16],
                        &self.system_palette,
                        &[palette_entry, palette_entry, palette_entry, palette_entry],
                    );
                }
            }
            self.palette_texture
                .update(None, &self.frame_buffer.data, self.frame_buffer.width * 3)
                .unwrap();
            front_end_state
                .palette_canvas
                .copy(&self.palette_texture, None, None)
                .unwrap();
            front_end_state.palette_canvas.present();
        }

        if front_end_state.actions.show_sprites {
            if let Some(color) = BACKGROUND_COLOR {
                self.frame_buffer.fill(color);
            } else {
                self.frame_buffer
                    .fill(ppu.get_universal_background_color(system_palette));
            }
            render_oam_table(ppu, system_palette, rom, &mut self.frame_buffer);
            self.sprite_texture
                .update(
                    Some(sdl2::rect::Rect::new(
                        0,
                        0,
                        SPRITE_TABLE_DIMENSIONS.0,
                        SPRITE_TABLE_DIMENSIONS.1,
                    )),
                    &self.frame_buffer.data,
                    self.frame_buffer.width * 3,
                )
                .unwrap();
            if let Some(color) = BACKGROUND_COLOR {
                self.frame_buffer.fill(color);
            } else {
                self.frame_buffer
                    .fill(ppu.get_universal_background_color(system_palette));
            }
            render_oam_with_pos(ppu, system_palette, rom, &mut self.frame_buffer);
            self.sprite_texture
                .update(
                    Some(sdl2::rect::Rect::new(
                        SPRITE_TABLE_DIMENSIONS.0 as i32,
                        0,
                        SCREEN_WIDTH as u32,
                        SCREEN_HEIGHT as u32,
                    )),
                    &self.frame_buffer.data,
                    self.frame_buffer.width * 3,
                )
                .unwrap();
            front_end_state
                .sprite_canvas
                .copy(&self.sprite_texture, None, None)
                .unwrap();

            if front_end_state.actions.show_grid {
                front_end_state
                    .sprite_canvas
                    .copy(
                        &self.sprite_view_grid_texture,
                        None,
                        Some(sdl2::rect::Rect::new(
                            SPRITE_TABLE_DIMENSIONS.0 as i32,
                            0,
                            SCREEN_WIDTH as u32,
                            SCREEN_HEIGHT as u32,
                        )),
                    )
                    .unwrap();
            }

            if front_end_state.actions.show_grid {
                front_end_state
                    .sprite_canvas
                    .copy(
                        &self.sprite_table_grid_texture,
                        None,
                        Some(sdl2::rect::Rect::new(
                            0,
                            0,
                            SPRITE_TABLE_DIMENSIONS.0,
                            SPRITE_TABLE_DIMENSIONS.1,
                        )),
                    )
                    .unwrap();
            }

            front_end_state.sprite_canvas.present();
        }

        if self.frame_counter.is_multiple_of(10u32) {
            let palette = [
                ppu.read_palette_table(0),
                ppu.read_palette_table(1),
                ppu.read_palette_table(2),
                ppu.read_palette_table(3),
            ];
            if front_end_state.actions.show_tiles {
                let num_tiles = rom.chr_rom_len() / 16;
                for i in 0..num_tiles {
                    self.frame_buffer
                        .render_tile((i % 32) * 8, (i / 32) * 8, rom, 0, i, &palette);
                }
                let num_rows = num_tiles * 8 / self.frame_buffer.width;
                self.tile_texture
                    .update(
                        Some(sdl2::rect::Rect::new(0, 0, 256, num_rows as u32 * 8)),
                        &self.frame_buffer.data,
                        self.frame_buffer.width * 3,
                    )
                    .unwrap();
                front_end_state
                    .tile_canvas
                    .copy(&self.tile_texture, None, None)
                    .unwrap();

                if front_end_state.actions.show_grid {
                    front_end_state
                        .tile_canvas
                        .copy(&self.tiles_grid_texture, None, None)
                        .unwrap();
                }

                front_end_state.tile_canvas.present();
            }
        }

        self.frame_counter += 1;
    }

    /// update textures related to the rewind UI
    fn update_rewind_ui(
        &mut self,
        front_end_state: &mut FrontEndState,
        rewind_buffer: &RingBuffer<(Frame, Vec<u8>), HISTORY_SIZE>,
    ) {
        if front_end_state.actions.rewind_move_left {
            front_end_state.actions.rewind_move_left = false;
            front_end_state.rewind_slot =
                (front_end_state.rewind_slot + 1).clamp(0, rewind_buffer.writer_head - 1);
        }
        if front_end_state.actions.rewind_move_right {
            front_end_state.actions.rewind_move_right = false;
            front_end_state.rewind_slot = front_end_state.rewind_slot.saturating_sub(1);
        }
        front_end_state
            .main_canvas
            .set_draw_color(Color::RGBA(0, 0, 0, 160));
        let _ = front_end_state.main_canvas.fill_rect(Rect::new(
            0,
            0,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        ));
        let main_slot = rewind_buffer.writer_head - (front_end_state.rewind_slot + 1);
        let rewind_frames: HashMap<i64, &Frame> = (-2i64..=2i64)
            .filter_map(|pos| {
                let slot = main_slot as i64 + pos;
                if !(0i64..(rewind_buffer.writer_head as i64)).contains(&slot) {
                    None
                } else {
                    rewind_buffer.peak(slot as usize).map(|e| (pos, &e.0))
                }
            })
            .collect();
        let rewind_frame_rects = [
            Rect::new(
                SCREEN_WIDTH as i32 / 4 - SCREEN_WIDTH as i32 / 8 - SCREEN_WIDTH as i32 / 16 - 16,
                SCREEN_HEIGHT as i32 / 2 + SCREEN_HEIGHT as i32 / 4 + SCREEN_HEIGHT as i32 / 16 + 8,
                SCREEN_WIDTH as u32 / 16,
                SCREEN_HEIGHT as u32 / 16,
            ),
            Rect::new(
                SCREEN_WIDTH as i32 / 4 - SCREEN_WIDTH as i32 / 8 - 8,
                SCREEN_HEIGHT as i32 / 2 + SCREEN_HEIGHT as i32 / 4,
                SCREEN_WIDTH as u32 / 8,
                SCREEN_HEIGHT as u32 / 8,
            ),
            Rect::new(
                SCREEN_WIDTH as i32 / 4,
                SCREEN_HEIGHT as i32 / 2 - 36,
                SCREEN_WIDTH as u32 / 2,
                SCREEN_HEIGHT as u32 / 2,
            ),
            Rect::new(
                SCREEN_WIDTH as i32 / 4 + SCREEN_WIDTH as i32 / 2 + 8,
                SCREEN_HEIGHT as i32 / 2 + SCREEN_HEIGHT as i32 / 4,
                SCREEN_WIDTH as u32 / 8,
                SCREEN_HEIGHT as u32 / 8,
            ),
            Rect::new(
                SCREEN_WIDTH as i32 / 4 + SCREEN_WIDTH as i32 / 2 + SCREEN_WIDTH as i32 / 8 + 16,
                SCREEN_HEIGHT as i32 / 2 + SCREEN_HEIGHT as i32 / 4 + SCREEN_HEIGHT as i32 / 16 + 8,
                SCREEN_WIDTH as u32 / 16,
                SCREEN_HEIGHT as u32 / 16,
            ),
        ];

        for (key, frame) in rewind_frames {
            self.rewind_texture
                .update(None, &frame.data, frame.width * 3)
                .unwrap();
            front_end_state
                .main_canvas
                .copy(
                    &self.rewind_texture,
                    None,
                    Some(rewind_frame_rects[(key + rewind_frame_rects.len() as i64 / 2) as usize]),
                )
                .unwrap();
        }
    }
}
