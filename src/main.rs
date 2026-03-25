use clap::Parser;
use hound::{WavSpec, WavWriter};
use itertools::Itertools;
use nes_rs::{
    bus::{ControllerCallback, GraphicsCallback, AUDIO_BUFFER_SIZE},
    controller::{Controller, ControllerButtons, ControllerInput},
    ppu::{palette::SystemPalette, PPU},
    rendering::{
        fps_frame::FPSFrame,
        frame::{Frame, SCREEN_HEIGHT, SCREEN_WIDTH, TILE_WIDTH},
        render_nametable, render_oam_table, render_oam_with_pos, write_tile,
    },
    ring_buffer::RingBuffer,
    rom::Rom,
    NES,
};
use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioSpecDesired},
    event::{Event, WindowEvent},
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    rect::{Point, Rect},
    render::{BlendMode, Canvas, ScaleMode, Texture, TextureCreator, WindowCanvas},
    video::{Window, WindowBuildError, WindowContext},
    AudioSubsystem, EventPump, Sdl, VideoSubsystem,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, File},
    io::{self, BufWriter, Read},
    ops::Neg,
    path::Path,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
    usize,
};

const PALETTE_VIEWER_DIMENSIONS: (u32, u32) = (4 * 8, 8 * 8);
const SPRITE_TABLE_DIMENSIONS: (u32, u32) = (8 * 8, 8 * 8);
const SPRITE_VIEW_DIMENSIONS: (u32, u32) = (SPRITE_TABLE_DIMENSIONS.0 + 256, 240);
const GRID_PIXEL_IN_NES_PIXEL: u32 = 2;
const FONT_NUMBERS_OFFSET: usize = 16;
const FONT_LETTERS_OFFSET: usize = 33;
const BACKGROUND_COLOR: Option<(u8, u8, u8)> = Some((0x66, 0x66, 0x66));

const HISTORY_SIZE: usize = 1800;
type RewindBuffer = RingBuffer<(Frame, Vec<u8>), HISTORY_SIZE>;

//TODO:
//reset rewind slot on esc or z exit
//xbox controller?

/// A NES emulator
#[derive(Parser, Debug)]
struct Args {
    /// whether to record the in game audio. The recoding is written to "./<rom_name>.wav"
    #[arg(long, default_value_t = false)]
    export_wav: bool,

    /// the scaling factor
    #[arg(long, default_value_t = 6)]
    scaling: u32,

    /// provide a path to a custom palette
    #[arg(long)]
    palette_path: Option<String>,

    /// enables integer scaling
    #[arg(long, default_value_t = false)]
    enable_integer_scaling: bool,

    /// path to the ROM
    rom_path: String,
}

struct AudioWrapper {
    last_sample: f32,
    #[allow(clippy::type_complexity)]
    func: Box<dyn FnMut(&mut f32, &mut [f32]) + Send>,
}

impl AudioCallback for AudioWrapper {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        (self.func)(&mut self.last_sample, out);
    }
}

type ConcurrentWavWriter = Option<Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>>;

struct AudioDeviceWrapper {
    audio_device: AudioDevice<AudioWrapper>,
    wav_writer: ConcurrentWavWriter,
}

impl AudioDeviceWrapper {
    fn new(
        front_end_state: &FrontEndState,
        audio_buffer: Arc<Mutex<RingBuffer<f32, AUDIO_BUFFER_SIZE>>>,
    ) -> Self {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: Some(1024),
        };

        let audio_device = front_end_state
            .audio_subsystem
            .open_playback(None, &desired_spec, |_spec| AudioWrapper {
                last_sample: 0f32,
                func: Box::new(move |last_sample, out: &mut [f32]| {
                    let mut buf = audio_buffer.lock().unwrap();
                    for x in out {
                        let sample = buf.next().unwrap_or(*last_sample);
                        *last_sample = sample;
                        *x = sample;
                    }
                }),
            })
            .unwrap();

        Self {
            audio_device,
            wav_writer: None,
        }
    }

    fn new_recording(
        front_end_state: &FrontEndState,
        output_path: String,
        audio_buffer: Arc<Mutex<RingBuffer<f32, AUDIO_BUFFER_SIZE>>>,
    ) -> Self {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: Some(1024),
        };

        let wav_spec = WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let wav = Arc::new(Mutex::new(Some(
            WavWriter::create(output_path, wav_spec).unwrap(),
        )));
        let wav_clone = wav.clone();

        let audio_device = front_end_state
            .audio_subsystem
            .open_playback(None, &desired_spec, |_spec| AudioWrapper {
                last_sample: 0f32,
                func: Box::new(move |last_sample, out: &mut [f32]| {
                    let mut buf = audio_buffer.lock().unwrap();
                    let mut wav = wav_clone.lock().unwrap();
                    for x in out {
                        let sample = buf.next().unwrap_or(*last_sample);
                        *last_sample = sample;
                        *x = sample;
                        let sample_i16 = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                        wav.as_mut().unwrap().write_sample(sample_i16).unwrap();
                    }
                }),
            })
            .unwrap();

        Self {
            audio_device,
            wav_writer: Some(wav),
        }
    }
}

fn create_window(
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

fn creates_canvas_and_texture_creator(
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

#[allow(dead_code)]
struct FrontEndState {
    sdl_context: Sdl,
    actions: Actions,
    audio_subsystem: AudioSubsystem,
    tile_map_canvas: WindowCanvas,
    main_canvas: WindowCanvas,
    tile_canvas: WindowCanvas,
    palette_canvas: WindowCanvas,
    sprite_canvas: WindowCanvas,
    event_pump: EventPump,
    key_map_1: HashMap<Keycode, ControllerButtons>,
    key_map_2: HashMap<Keycode, ControllerButtons>,
    nes_controller_input: Vec<ControllerInput>,
    rewind_slot: usize,
}

struct TextureCreators {
    main_creator: TextureCreator<WindowContext>,
    tile_map_creator: TextureCreator<WindowContext>,
    tile_creator: TextureCreator<WindowContext>,
    palette_creator: TextureCreator<WindowContext>,
    sprite_creator: TextureCreator<WindowContext>,
}

struct Textures<'a> {
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

fn create_grid_texture<'a>(
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
    fn new(
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

    fn update_textures(
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
        if front_end_state.actions.rewind_mode {
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
            let rewind_frame_rects = vec![
                Rect::new(
                    SCREEN_WIDTH as i32 / 4
                        - SCREEN_WIDTH as i32 / 8
                        - SCREEN_WIDTH as i32 / 16
                        - 16,
                    SCREEN_HEIGHT as i32 / 2
                        + SCREEN_HEIGHT as i32 / 4
                        + SCREEN_HEIGHT as i32 / 16
                        + 8,
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
                    SCREEN_WIDTH as i32 / 4
                        + SCREEN_WIDTH as i32 / 2
                        + SCREEN_WIDTH as i32 / 8
                        + 16,
                    SCREEN_HEIGHT as i32 / 2
                        + SCREEN_HEIGHT as i32 / 4
                        + SCREEN_HEIGHT as i32 / 16
                        + 8,
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
                        Some(
                            rewind_frame_rects
                                [(key + rewind_frame_rects.len() as i64 / 2) as usize],
                        ),
                    )
                    .unwrap();
            }
        }

        if front_end_state.actions.show_grid {
            front_end_state
                .main_canvas
                .copy(&self.main_screen_grid_texture, None, None)
                .unwrap();
        }

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
}

impl FrontEndState {
    fn new(rom_name: &str, scaling: u32, integer_scaling: bool) -> (Self, TextureCreators) {
        let sdl_context = sdl2::init().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = create_window(
            &video_subsystem,
            &format!("NESrs -- {rom_name}"),
            SCREEN_WIDTH as u32 * scaling,
            SCREEN_HEIGHT as u32 * scaling,
        )
        .unwrap();

        let window_tile_map = create_window(
            &video_subsystem,
            &format!("Tile Map NESrs -- {rom_name}"),
            SCREEN_WIDTH as u32 * scaling,
            SCREEN_HEIGHT as u32 * scaling,
        )
        .unwrap();

        let window_tile = create_window(
            &video_subsystem,
            &format!("Tiles NESrs -- {rom_name}"),
            SCREEN_WIDTH as u32 * scaling,
            SCREEN_HEIGHT as u32 * scaling,
        )
        .unwrap();

        let window_palette = create_window(
            &video_subsystem,
            &format!("Palette NESrs -- {rom_name}"),
            PALETTE_VIEWER_DIMENSIONS.0 * scaling,
            PALETTE_VIEWER_DIMENSIONS.1 * scaling,
        )
        .unwrap();

        let window_sprite = create_window(
            &video_subsystem,
            &format!("Sprite NESrs -- {rom_name}"),
            SPRITE_VIEW_DIMENSIONS.0 * scaling,
            SPRITE_VIEW_DIMENSIONS.1 * scaling,
        )
        .unwrap();

        let (tile_map_canvas, tile_map_creator) = creates_canvas_and_texture_creator(
            window_tile_map,
            SCREEN_WIDTH as u32 * 2 * 2,
            SCREEN_HEIGHT as u32 * 2 * 2,
            integer_scaling,
        );

        let (tile_canvas, tile_creator) = creates_canvas_and_texture_creator(
            window_tile,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
            integer_scaling,
        );

        let (mut main_canvas, main_creator) = creates_canvas_and_texture_creator(
            window,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
            integer_scaling,
        );
        main_canvas.set_blend_mode(BlendMode::Blend);

        let (palette_canvas, palette_creator) = creates_canvas_and_texture_creator(
            window_palette,
            PALETTE_VIEWER_DIMENSIONS.0,
            PALETTE_VIEWER_DIMENSIONS.1,
            integer_scaling,
        );

        let (sprite_canvas, sprite_creator) = creates_canvas_and_texture_creator(
            window_sprite,
            SPRITE_VIEW_DIMENSIONS.0,
            SPRITE_VIEW_DIMENSIONS.1,
            integer_scaling,
        );

        let event_pump = sdl_context.event_pump().unwrap();

        let mut key_map_1 = HashMap::new();
        key_map_1.insert(Keycode::Down, ControllerButtons::DOWN);
        key_map_1.insert(Keycode::Up, ControllerButtons::UP);
        key_map_1.insert(Keycode::Right, ControllerButtons::RIGHT);
        key_map_1.insert(Keycode::Left, ControllerButtons::LEFT);
        key_map_1.insert(Keycode::A, ControllerButtons::A);
        key_map_1.insert(Keycode::S, ControllerButtons::B);
        key_map_1.insert(Keycode::Space, ControllerButtons::SELECT);
        key_map_1.insert(Keycode::Return, ControllerButtons::START);

        let mut key_map_2 = HashMap::new();
        key_map_2.insert(Keycode::J, ControllerButtons::DOWN);
        key_map_2.insert(Keycode::K, ControllerButtons::UP);
        key_map_2.insert(Keycode::L, ControllerButtons::RIGHT);
        key_map_2.insert(Keycode::H, ControllerButtons::LEFT);
        key_map_2.insert(Keycode::U, ControllerButtons::A);
        key_map_2.insert(Keycode::I, ControllerButtons::B);
        key_map_2.insert(Keycode::O, ControllerButtons::SELECT);
        key_map_2.insert(Keycode::P, ControllerButtons::START);

        (
            Self {
                sdl_context,
                actions: Actions::new(),
                audio_subsystem,
                tile_map_canvas,
                main_canvas,
                tile_canvas,
                palette_canvas,
                sprite_canvas,
                event_pump,
                key_map_1,
                key_map_2,
                nes_controller_input: Vec::new(),
                rewind_slot: 0,
            },
            TextureCreators {
                main_creator,
                tile_map_creator,
                tile_creator,
                palette_creator,
                sprite_creator,
            },
        )
    }

    fn show_active_windows(&mut self) {
        if self.actions.show_tile_map {
            self.tile_map_canvas.window_mut().show();
        } else {
            self.tile_map_canvas.window_mut().hide();
        }

        if self.actions.show_tiles {
            self.tile_canvas.window_mut().show();
        } else {
            self.tile_canvas.window_mut().hide();
        }

        if self.actions.show_palette {
            self.palette_canvas.window_mut().show();
        } else {
            self.palette_canvas.window_mut().hide();
        }

        if self.actions.show_sprites {
            self.sprite_canvas.window_mut().show();
        } else {
            self.sprite_canvas.window_mut().hide();
        }
    }
}

struct Actions {
    should_quit: bool,
    pause: bool,
    show_tile_map: bool,
    show_tiles: bool,
    show_palette: bool,
    show_sprites: bool,
    show_fps: bool,
    show_grid: bool,
    save_state: bool,
    load_state: bool,
    speed_multiplier: f64,
    rewind_mode: bool,
    rewind_move_left: bool,
    rewind_move_right: bool,
    rewind_load_slot: Option<usize>,
}

impl Actions {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            pause: false,
            show_tile_map: false,
            show_tiles: false,
            show_palette: false,
            show_sprites: false,
            show_fps: false,
            show_grid: false,
            save_state: false,
            load_state: false,
            speed_multiplier: 1f64,
            rewind_mode: false,
            rewind_move_left: false,
            rewind_move_right: false,
            rewind_load_slot: None,
        }
    }
}

fn main() {
    let args = Args::parse();
    let rom_path = args.rom_path;
    let rom_name = Path::new(&rom_path)
        .file_name()
        .iter()
        .filter_map(|n| {
            let name = n.to_str()?;
            name.split('.').dropping_back(1).next_back()
        })
        .next_back()
        .unwrap_or("rom");
    let save_state_path = Path::new(&rom_path).with_extension("save_state");
    let palette_path = args.palette_path;

    let (front_end_state, texture_creators) =
        FrontEndState::new(rom_name, args.scaling, args.enable_integer_scaling);
    let front_end_state = Rc::new(RefCell::new(front_end_state));

    let mut rewind_buffer = Rc::new(RefCell::new(RewindBuffer::new()));

    let rom = Rom::load_from_disk(&rom_path).unwrap();

    let palette = if let Some(path) = palette_path {
        read_palette_table(&path).unwrap_or_default()
    } else {
        SystemPalette::new()
    };

    let font_chr_rom = include_bytes!("../om_thick_plain_nes.chr");

    let (render_frame, handle_controller_input) = create_callbacks(
        front_end_state.clone(),
        palette.clone(),
        &texture_creators,
        font_chr_rom,
        rewind_buffer.clone(),
    );

    let (mut nes, audio_buffer) = NES::new(
        rom,
        palette.clone(),
        1f64,
        render_frame,
        handle_controller_input,
    );

    let audio_device_wrapper = if args.export_wav {
        AudioDeviceWrapper::new_recording(
            &front_end_state.borrow(),
            format!("{rom_name}.wav"),
            audio_buffer.clone(),
        )
    } else {
        AudioDeviceWrapper::new(&front_end_state.borrow(), audio_buffer.clone())
    };

    audio_device_wrapper.audio_device.resume();
    let mut last_speed = 1f64;
    let mut next_save_on_frame = 0;
    while !front_end_state.borrow().actions.should_quit {
        let pause = front_end_state.borrow().actions.pause;
        handle_user_input(&mut front_end_state.borrow_mut());

        front_end_state.borrow_mut().show_active_windows();

        if last_speed != front_end_state.borrow().actions.speed_multiplier {
            last_speed = front_end_state.borrow().actions.speed_multiplier;
            nes.set_speed_multiplayer(last_speed);
        }

        if front_end_state.borrow().actions.save_state {
            let save = match create_save_state_bin(&nes) {
                Ok(save) => save,
                Err(err) => {
                    eprintln!("Failed to create save state: {err}");
                    Vec::new()
                }
            };
            if !save.is_empty() {
                if let Err(err) = fs::write(&save_state_path, save) {
                    eprintln!("Failed to write save state: {err}");
                }
            }
            front_end_state.borrow_mut().actions.save_state = false;
        }

        if front_end_state.borrow().actions.load_state {
            let save = match fs::read(&save_state_path) {
                Ok(save) => save,
                Err(err) => {
                    eprintln!("Failed to load save state: {err}");
                    Vec::new()
                }
            };
            if !save.is_empty() {
                if let Some(old_state) = resume_from_save_state_bin(nes, &save) {
                    nes = old_state;
                    nes.manual_re_render();
                } else {
                    panic!("Failed to resume from save state.")
                }
            }
            front_end_state.borrow_mut().actions.load_state = false;
            next_save_on_frame = nes.get_frame_counter() + 10;
        }

        if next_save_on_frame == nes.get_frame_counter() {
            next_save_on_frame += 10;
            rewind_buffer.borrow_mut().push((
                nes.get_current_frame(),
                create_save_state_bin(&nes).unwrap(),
            ));
        }

        {
            let mut front_end_state_bo = front_end_state.borrow_mut();
            if let Some(rewind_slot) = front_end_state_bo.actions.rewind_load_slot {
                let rewind_slot = rewind_buffer
                    .borrow()
                    .writer_head
                    .saturating_sub(rewind_slot + 1);
                let (_, state) = rewind_buffer.borrow_mut().get(rewind_slot).unwrap();
                nes = resume_from_save_state_bin(nes, &state).unwrap();
                front_end_state_bo.rewind_slot = 0;
                front_end_state_bo.actions.rewind_load_slot = None;
                front_end_state_bo.actions.rewind_mode = false;
                front_end_state_bo.actions.pause = false;
                next_save_on_frame = nes.get_frame_counter() + 10;
            }
        }

        if !pause {
            for _ in 0..1000 {
                nes.step();
            }
        } else {
            thread::sleep(Duration::from_millis(16)); // Roughly 60FPS avoids wasting resources
                                                      // when the emulation is paused
            nes.manual_re_render(); // without this windows such as the tile map would only show
                                    // once the emulation gets resumed
        }
    }
    drop(audio_device_wrapper.audio_device);
    let writer = {
        if let Some(writer) = audio_device_wrapper.wav_writer {
            let mut guard = writer.lock().unwrap();
            guard.take()
        } else {
            None
        }
    };
    if let Some(writer) = writer {
        writer.finalize().unwrap();
    }
}

//TODO: implement rewind with preview image every ~10 frames

//TODO: maybe use bitcode with encode decode bitcode + serde is slower
fn create_save_state_bin(nes: &NES) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_stdvec(&nes)
}

#[allow(dead_code)]
fn create_save_state_json(nes: &NES) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&nes)
}

fn resume_from_save_state_bin<'a>(nes: NES<'a>, data: &[u8]) -> Option<NES<'a>> {
    nes.replace_state(postcard::from_bytes(data).ok()?)
}

#[allow(dead_code)]
fn resume_from_save_state_json<'a>(nes: NES<'a>, data: &str) -> Option<NES<'a>> {
    nes.replace_state(serde_json::from_str(data).ok()?)
}

fn create_callbacks<'a>(
    front_end_state: Rc<RefCell<FrontEndState>>,
    palette: SystemPalette,
    texture_creators: &'a TextureCreators,
    font_chr_rom: &'static [u8],
    rewind_buffer: Rc<RefCell<RewindBuffer>>,
) -> (impl GraphicsCallback<'a>, impl ControllerCallback<'a>) {
    let front_end_state_controller = front_end_state.clone();
    let front_end_state_rendering = front_end_state.clone();

    let handle_controller_input =
        move |controller_1: &mut Controller, controller_2: &mut Controller| {
            let mut front_end = front_end_state_controller.borrow_mut();
            for button in front_end.nes_controller_input.drain(0..) {
                match button {
                    ControllerInput::Controller1(pressed, key) => {
                        controller_1.set_button_state(pressed, key)
                    }
                    ControllerInput::Controller2(pressed, key) => {
                        controller_2.set_button_state(pressed, key)
                    }
                }
            }
        };

    let mut textures = Textures::new(
        &mut front_end_state.borrow_mut(),
        texture_creators,
        palette.clone(),
    );

    let render_frame = move |ppu: &PPU, frame: &Frame, fps: u32, rom: &Rom| {
        textures.update_textures(
            frame,
            fps,
            &mut front_end_state_rendering.borrow_mut(),
            ppu,
            rom,
            font_chr_rom,
            &palette,
            &rewind_buffer.borrow(),
        );
        // front_end_state_rendering.borrow_mut().actions.pause = true;

        //TODO: add screenshot function and maybe video recoding
        // image::save_buffer(
        //     format!("./{rom_name}.png"),
        //     &frame.data,
        //     SCREEN_WIDTH as u32,
        //     SCREEN_HEIGHT as u32,
        //     image::ColorType::Rgb8,
        // )
        // .unwrap();
    };
    (render_frame, handle_controller_input)
}

fn handle_user_input(front_end: &mut FrontEndState) {
    while let Some(event) = front_end.event_pump.poll_event() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Q),
                ..
            } => front_end.actions.should_quit = true,
            Event::Window {
                timestamp: _,
                window_id,
                win_event: WindowEvent::Close,
            } => {
                if front_end.main_canvas.window().id() == window_id {
                    front_end.actions.should_quit = true;
                } else if front_end.tile_canvas.window().id() == window_id {
                    front_end.actions.show_tiles = false;
                } else if front_end.tile_map_canvas.window().id() == window_id {
                    front_end.actions.show_tile_map = false;
                } else if front_end.sprite_canvas.window().id() == window_id {
                    front_end.actions.show_sprites = false;
                } else if front_end.palette_canvas.window().id() == window_id {
                    front_end.actions.show_palette = false;
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::M),
                ..
            } => front_end.actions.show_tile_map ^= true,
            Event::KeyDown {
                keycode: Some(Keycode::T),
                ..
            } => front_end.actions.show_tiles ^= true,
            Event::KeyDown {
                keycode: Some(Keycode::P),
                ..
            } => front_end.actions.show_palette ^= true,
            Event::KeyDown {
                keycode: Some(Keycode::O),
                ..
            } => front_end.actions.show_sprites ^= true,
            Event::KeyDown {
                keycode: Some(Keycode::F),
                ..
            } => front_end.actions.show_fps ^= true,
            Event::KeyDown {
                keycode: Some(Keycode::G),
                ..
            } => front_end.actions.show_grid ^= true,
            Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                front_end.actions.pause ^= true;
                if front_end.actions.rewind_mode && !front_end.actions.pause {
                    front_end.actions.rewind_mode = false;
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::E),
                ..
            } => front_end.actions.save_state = true,
            Event::KeyDown {
                keycode: Some(Keycode::R),
                ..
            } => front_end.actions.load_state = true,
            Event::KeyDown {
                keycode: Some(Keycode::Z),
                ..
            } => {
                front_end.actions.rewind_mode ^= true;
                front_end.actions.pause = front_end.actions.rewind_mode;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Plus),
                ..
            } => {
                if front_end.actions.speed_multiplier < 1f64 {
                    front_end.actions.speed_multiplier *= 2f64;
                } else {
                    front_end.actions.speed_multiplier += 1f64;
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::Minus),
                ..
            } => {
                if front_end.actions.speed_multiplier <= 1f64 {
                    front_end.actions.speed_multiplier /= 2f64;
                } else {
                    front_end.actions.speed_multiplier -= 1f64;
                }
                front_end.actions.speed_multiplier =
                    front_end.actions.speed_multiplier.clamp(0.01f64, 50f64);
            }
            Event::KeyDown {
                keycode: Some(keycode),
                ..
            } => {
                if front_end.actions.rewind_mode && keycode == Keycode::Left {
                    front_end.actions.rewind_move_left = true;
                } else if front_end.actions.rewind_mode && keycode == Keycode::Right {
                    front_end.actions.rewind_move_right = true;
                } else if front_end.actions.rewind_mode && keycode == Keycode::Space {
                    front_end.actions.rewind_load_slot = Some(front_end.rewind_slot)
                }

                if let Some(&key) = front_end.key_map_1.get(&keycode) {
                    front_end
                        .nes_controller_input
                        .push(ControllerInput::Controller1(true, key));
                } else if let Some(&key) = front_end.key_map_2.get(&keycode) {
                    front_end
                        .nes_controller_input
                        .push(ControllerInput::Controller2(true, key));
                }
            }
            Event::KeyUp {
                keycode: Some(keycode),
                ..
            } => {
                if front_end.actions.rewind_mode && keycode == Keycode::Left {
                } else if front_end.actions.rewind_mode && keycode == Keycode::Right {
                } else if front_end.actions.rewind_mode && keycode == Keycode::Space {
                }
                if let Some(&key) = front_end.key_map_1.get(&keycode) {
                    front_end
                        .nes_controller_input
                        .push(ControllerInput::Controller1(false, key));
                } else if let Some(&key) = front_end.key_map_2.get(&keycode) {
                    front_end
                        .nes_controller_input
                        .push(ControllerInput::Controller2(false, key));
                }
            }
            _ => { /* do nothing */ }
        }
    }
}

fn read_palette_table(path: &str) -> io::Result<SystemPalette> {
    let mut palette_file = File::open(path)?;
    let mut buffer = Vec::new();
    palette_file.read_to_end(&mut buffer)?;
    Ok(SystemPalette::from_raw(&buffer).unwrap())
}
