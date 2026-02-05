use clap::Parser;
use hound::{WavSpec, WavWriter};
use itertools::Itertools;
use nes_rs::bus::Bus;
use nes_rs::bus::AUDIO_BUFFER_SIZE;
use nes_rs::controller::{Controller, ControllerButtons, ControllerInput};
use nes_rs::cpu::CPU;
use nes_rs::ppu::palette::SystemPalette;
use nes_rs::ppu::PPU;
use nes_rs::rendering::fps_frame::FPSFrame;
use nes_rs::rendering::frame::Frame;
use nes_rs::rendering::frame::SCREEN_HEIGHT;
use nes_rs::rendering::frame::SCREEN_WIDTH;
use nes_rs::rendering::render_nametable;
use nes_rs::rendering::render_oam_table;
use nes_rs::rendering::render_oam_with_pos;
use nes_rs::rendering::write_tile;
use nes_rs::ring_buffer::RingBuffer;
use nes_rs::rom::Rom;
use sdl2::audio::AudioDevice;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::Window;
use sdl2::video::WindowBuildError;
use sdl2::video::WindowContext;
use sdl2::VideoSubsystem;
use sdl2::{AudioSubsystem, EventPump, Sdl};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

//TODO: split crate in core and front end and add old front end as minimal example
//TODO: avoid unwrap

//TODO: Debug features
// - save state

const PALETTE_VIEWER_DIMENSIONS: (u32, u32) = (4 * 8, 8 * 8);
const SPRITE_TABLE_DIMENSIONS: (u32, u32) = (8 * 8, 8 * 8);
const SPRITE_VIEW_DIMENSIONS: (u32, u32) = (SPRITE_TABLE_DIMENSIONS.0 + 256, 240);

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
    #[allow(clippy::type_complexity)]
    func: Box<dyn FnMut(&mut [f32]) + Send>,
}

impl AudioCallback for AudioWrapper {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        (self.func)(out);
    }
}

struct AudioDeviceWrapper {
    audio_device: AudioDevice<AudioWrapper>,
    wav_writer: Option<Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>>,
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
                func: Box::new(move |out: &mut [f32]| {
                    let mut buf = audio_buffer.lock().unwrap();
                    for x in out {
                        let sample = buf.next().unwrap_or(0f32);
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
                func: Box::new(move |out: &mut [f32]| {
                    let mut buf = audio_buffer.lock().unwrap();
                    let mut wav = wav_clone.lock().unwrap();
                    for x in out {
                        let sample = buf.next().unwrap_or(0f32);
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
    fps_texture: Texture<'a>,
    tile_texture: Texture<'a>,
    palette_texture: Texture<'a>,
    sprite_texture: Texture<'a>,
    nametable_textures: Vec<Texture<'a>>,

    frame_buffer: Frame,
    frame_counter: u32,
    system_palette: SystemPalette,
}

impl<'a> Textures<'a> {
    fn new(texture_creators: &'a TextureCreators, system_palette: SystemPalette) -> Textures<'a> {
        let main_texture = texture_creators
            .main_creator
            .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
            .unwrap();

        let fps_texture = texture_creators
            .main_creator
            .create_texture_target(PixelFormatEnum::RGB24, 48, 8)
            .unwrap();

        let tile_texture = texture_creators
            .tile_creator
            .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
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
        Textures {
            main_texture,
            fps_texture,
            tile_texture,
            palette_texture,
            sprite_texture,
            nametable_textures,
            frame_buffer: Frame::default(),
            system_palette,
            frame_counter: 0,
        }
    }

    fn update_textures(
        &mut self,
        emulation_frame: &Frame,
        fps_frame: &Frame,
        front_end_state: &mut FrontEndState,
        ppu: &PPU,
        rom: &Rom,
    ) {
        self.main_texture
            .update(None, &emulation_frame.data, emulation_frame.width * 3)
            .unwrap();
        front_end_state
            .main_canvas
            .copy(&self.main_texture, None, None)
            .unwrap();

        if front_end_state.actions.show_fps {
            self.fps_texture
                .update(None, &fps_frame.data, fps_frame.width * 3)
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
                    render_nametable(ppu, &rom, i, &mut self.frame_buffer, &self.system_palette);
                    texture
                        .update(None, &self.frame_buffer.data, self.frame_buffer.width * 3)
                        .unwrap()
                });

            self.nametable_textures
                .iter()
                .enumerate()
                .for_each(|(i, texture)| {
                    let i = i as i32;
                    let x: i32 = i % 2 * 256;
                    let y: i32 = if i < 2 { 0 } else { 240 };
                    front_end_state
                        .tile_map_canvas
                        .copy(&texture, None, Some(sdl2::rect::Rect::new(x, y, 256, 240)))
                        .unwrap()
                });
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
            self.frame_buffer
                .fill(ppu.get_color_from_current_system_palette(
                    ppu.get_universal_background_color() as usize,
                ));
            render_oam_table(ppu, rom, &mut self.frame_buffer);
            self.sprite_texture
                .update(
                    Some(sdl2::rect::Rect::new(
                        0,
                        0,
                        SPRITE_TABLE_DIMENSIONS.0,
                        SPRITE_TABLE_DIMENSIONS.0,
                    )),
                    &self.frame_buffer.data,
                    self.frame_buffer.width * 3,
                )
                .unwrap();
            self.frame_buffer
                .fill(ppu.get_color_from_current_system_palette(
                    ppu.get_universal_background_color() as usize,
                ));
            render_oam_with_pos(ppu, rom, &mut self.frame_buffer);
            self.sprite_texture
                .update(
                    Some(sdl2::rect::Rect::new(
                        SPRITE_TABLE_DIMENSIONS.0 as i32,
                        0,
                        256,
                        240,
                    )),
                    &self.frame_buffer.data,
                    self.frame_buffer.width * 3,
                )
                .unwrap();
            front_end_state
                .sprite_canvas
                .copy(&self.sprite_texture, None, None)
                .unwrap();
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
                        .render_tile((i % 32) * 8, (i / 32) * 8, &rom, 0, i, &palette);
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

        let (tile_map_canvas, tile_map_creator) =
            creates_canvas_and_texture_creator(window_tile_map, 256 * 2, 240 * 2, integer_scaling);

        let (tile_canvas, tile_creator) =
            creates_canvas_and_texture_creator(window_tile, 256, 240, integer_scaling);

        let (main_canvas, main_creator) =
            creates_canvas_and_texture_creator(window, 256, 240, integer_scaling);

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
    save_state: bool,
    load_state: bool,
    speed_multiplier: f64,
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
            save_state: false,
            load_state: false,
            speed_multiplier: 1f64,
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
    let palette_path = args.palette_path;

    let (front_end_state, texture_creators) =
        FrontEndState::new(rom_name, args.scaling, args.enable_integer_scaling);
    let front_end_state = Rc::new(RefCell::new(front_end_state));
    let front_end_state_controller = front_end_state.clone();
    let front_end_state_rendering = front_end_state.clone();

    let bytes: Vec<u8> = std::fs::read(&rom_path).unwrap();
    let rom = Rom::new(&bytes).unwrap();

    let palette = if let Some(path) = palette_path {
        read_palette_table(&path).unwrap_or_default()
    } else {
        SystemPalette::new()
    };

    let mut textures = Textures::new(&texture_creators, palette.clone());

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

    //TODO: move fps rendering out of bus
    let render_frame = move |ppu: &PPU, frame: &Frame, fps_frame: &FPSFrame, rom: &Rom| {
        textures.update_textures(
            frame,
            &fps_frame.frame,
            &mut front_end_state_rendering.borrow_mut(),
            ppu,
            rom,
        );
    };

    let bus = Bus::new(rom, palette, 1f64, render_frame, handle_controller_input);

    let audio_buffer = bus.audio_ring_buffer.clone();
    let audio_device_wrapper = if args.export_wav {
        AudioDeviceWrapper::new_recording(
            &front_end_state.borrow(),
            format!("{rom_name}.wav"),
            audio_buffer,
        )
    } else {
        AudioDeviceWrapper::new(&front_end_state.borrow(), audio_buffer)
    };

    let mut cpu = CPU::new_with_bus(bus);
    audio_device_wrapper.audio_device.resume();
    cpu.reset();
    let mut last_speed = 1f64;
    while !front_end_state.borrow().actions.should_quit {
        let pause = front_end_state.borrow().actions.pause;
        handle_user_input(&mut front_end_state.borrow_mut());

        front_end_state.borrow_mut().show_active_windows();

        //TODO: would be nicer to have a nes struct instead of doing this on the CPU
        if last_speed != front_end_state.borrow().actions.speed_multiplier {
            last_speed = front_end_state.borrow().actions.speed_multiplier;
            cpu.set_speed_multiplayer(last_speed);
        }

        if !pause {
            for _ in 0..1000 {
                cpu.step();
            }
        } else {
            thread::sleep(Duration::from_millis(16)); // Roughly 60FPS avoids wasting resources
                                                      // when the emulation is paused
            cpu.manuel_re_render(); // without this windows such as the tile map would only show
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

fn handle_user_input(front_end: &mut FrontEndState) {
    while let Some(event) = front_end.event_pump.poll_event() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Q),
                ..
            } => front_end.actions.should_quit = true,
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
                keycode: Some(Keycode::Escape),
                ..
            } => front_end.actions.pause ^= true,
            Event::KeyDown {
                keycode: Some(Keycode::Plus),
                ..
            } => front_end.actions.speed_multiplier += 1f64,
            Event::KeyDown {
                keycode: Some(Keycode::Minus),
                ..
            } => {
                front_end.actions.speed_multiplier -= 1f64;
                front_end.actions.speed_multiplier =
                    front_end.actions.speed_multiplier.clamp(1f64, 50f64);
            }
            Event::KeyDown {
                keycode: Some(keycode),
                ..
            } => {
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
