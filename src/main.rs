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
use nes_rs::rendering::render_nametable;
use nes_rs::rendering::render_oam_table;
use nes_rs::rendering::render_oam_with_pos;
use nes_rs::rendering::write_tile;
use nes_rs::ring_buffer::RingBuffer;
use nes_rs::rom::Rom;
use num::Integer;
use sdl2::audio::AudioDevice;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
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

//TODO: Debug features
// - sprite viewer
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

impl FrontEndState {
    fn new(
        rom_name: &str,
        scaling: u32,
    ) -> (
        Self,
        TextureCreator<WindowContext>,
        TextureCreator<WindowContext>,
        TextureCreator<WindowContext>,
        TextureCreator<WindowContext>,
        TextureCreator<WindowContext>,
    ) {
        let sdl_context = sdl2::init().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(
                &format!("NESrs -- {rom_name}"),
                256 * scaling,
                240 * scaling,
            )
            .position_centered()
            .build()
            .unwrap();

        let window_tile_map = video_subsystem
            .window(
                &format!("Tile Map NESrs -- {rom_name}"),
                256 * scaling,
                240 * scaling,
            )
            .position_centered()
            .build()
            .unwrap();

        let window_tile = video_subsystem
            .window(
                &format!("Tiles NESrs -- {rom_name}"),
                256 * scaling,
                240 * scaling,
            )
            .position_centered()
            .build()
            .unwrap();

        let window_palette = video_subsystem
            .window(
                &format!("Palette NESrs -- {rom_name}"),
                PALETTE_VIEWER_DIMENSIONS.0 * scaling,
                PALETTE_VIEWER_DIMENSIONS.1 * scaling,
            )
            .position_centered()
            .build()
            .unwrap();

        let window_sprite = video_subsystem
            .window(
                &format!("Sprite NESrs -- {rom_name}"),
                SPRITE_VIEW_DIMENSIONS.0 * scaling,
                SPRITE_VIEW_DIMENSIONS.1 * scaling,
            )
            .position_centered()
            .build()
            .unwrap();

        let mut tile_map_canvas = window_tile_map.into_canvas().build().unwrap();
        tile_map_canvas
            .set_scale(scaling as f32 / 2f32, scaling as f32 / 2f32)
            .unwrap();
        let tile_map_creator = tile_map_canvas.texture_creator();

        let mut tile_canvas = window_tile.into_canvas().build().unwrap();
        tile_canvas
            .set_scale(scaling as f32, scaling as f32)
            .unwrap();
        let tile_creator = tile_canvas.texture_creator();

        let mut main_canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();
        main_canvas
            .set_scale(scaling as f32, scaling as f32)
            .unwrap();
        let main_creator = main_canvas.texture_creator();

        let mut palette_canvas = window_palette.into_canvas().build().unwrap();
        palette_canvas
            .set_scale(scaling as f32, scaling as f32)
            .unwrap();
        let palette_creator = palette_canvas.texture_creator();

        let mut sprite_canvas = window_sprite.into_canvas().build().unwrap();
        sprite_canvas
            .set_scale(scaling as f32, scaling as f32)
            .unwrap();
        let sprite_creator = sprite_canvas.texture_creator();

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
            main_creator,
            tile_map_creator,
            tile_creator,
            palette_creator,
            sprite_creator,
        )
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

    let (
        front_end_state,
        main_creator,
        tile_map_creator,
        tile_creator,
        palette_creator,
        sprite_creator,
    ) = FrontEndState::new(rom_name, args.scaling);
    let front_end_state = Rc::new(RefCell::new(front_end_state));
    let front_end_state_controller = front_end_state.clone();
    let front_end_state_rendering = front_end_state.clone();

    let mut main_texture = main_creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let mut fps_texture = main_creator
        .create_texture_target(PixelFormatEnum::RGB24, 48, 8)
        .unwrap();

    let mut tile_texture = tile_creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let mut palette_texture = palette_creator
        .create_texture_target(
            PixelFormatEnum::RGB24,
            PALETTE_VIEWER_DIMENSIONS.0,
            PALETTE_VIEWER_DIMENSIONS.1,
        )
        .unwrap();

    let mut sprite_texture = sprite_creator
        .create_texture_target(
            PixelFormatEnum::RGB24,
            SPRITE_VIEW_DIMENSIONS.0,
            SPRITE_VIEW_DIMENSIONS.1,
        )
        .unwrap();

    let mut nametable_textures = Vec::new();
    for _ in 0..4 {
        nametable_textures.push(
            tile_map_creator
                .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
                .unwrap(),
        );
    }

    let bytes: Vec<u8> = std::fs::read(&rom_path).unwrap();
    let rom = Rom::new(&bytes).unwrap();

    let palette = if let Some(path) = palette_path {
        read_palette_table(&path).unwrap_or_default()
    } else {
        SystemPalette::new()
    };

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

    let mut tmp_frame = Frame::default();
    let mut frame_counter = 0;
    let system_palette = SystemPalette::new();

    //TODO: move fps rendering out of bus
    let render_frame = move |ppu: &PPU, frame: &Frame, fps_frame: &FPSFrame, rom: &Rom| {
        main_texture
            .update(None, &frame.data, frame.width * 3)
            .unwrap();
        front_end_state_rendering
            .borrow_mut()
            .main_canvas
            .copy(&main_texture, None, None)
            .unwrap();

        if front_end_state_rendering.borrow().actions.show_fps {
            fps_texture
                .update(None, &fps_frame.frame.data, fps_frame.frame.width * 3)
                .unwrap();
            front_end_state_rendering
                .borrow_mut()
                .main_canvas
                .copy(&fps_texture, None, Some(sdl2::rect::Rect::new(5, 5, 48, 8)))
                .unwrap();
        }

        front_end_state_rendering.borrow_mut().main_canvas.present();

        if front_end_state_rendering.borrow().actions.show_tile_map {
            nametable_textures
                .iter_mut()
                .enumerate()
                .for_each(|(i, texture)| {
                    render_nametable(ppu, &rom, i, &mut tmp_frame, &system_palette);
                    texture
                        .update(None, &tmp_frame.data, tmp_frame.width * 3)
                        .unwrap()
                });

            nametable_textures
                .iter()
                .enumerate()
                .for_each(|(i, texture)| {
                    let i = i as i32;
                    let x: i32 = i % 2 * 256;
                    let y: i32 = if i < 2 { 0 } else { 240 };
                    front_end_state_rendering
                        .borrow_mut()
                        .tile_map_canvas
                        .copy(&texture, None, Some(sdl2::rect::Rect::new(x, y, 256, 240)))
                        .unwrap()
                });
            front_end_state_rendering
                .borrow_mut()
                .tile_map_canvas
                .present();
        }

        if front_end_state_rendering.borrow().actions.show_palette {
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
                        &mut tmp_frame,
                        color_idx * 8,
                        palette_idx * 8,
                        &[0u8; 16],
                        &system_palette,
                        &[palette_entry, palette_entry, palette_entry, palette_entry],
                    );
                }
            }
            palette_texture
                .update(
                    None,
                    &tmp_frame.data[0..tmp_frame.width * 3],
                    tmp_frame.width * 3,
                )
                .unwrap();
            front_end_state_rendering
                .borrow_mut()
                .palette_canvas
                .copy(&palette_texture, None, None)
                .unwrap();
            front_end_state_rendering
                .borrow_mut()
                .palette_canvas
                .present();
        }

        if front_end_state_rendering.borrow().actions.show_sprites {
            tmp_frame.fill(ppu.get_color_from_current_system_palette(
                ppu.get_universal_background_color() as usize,
            ));
            render_oam_table(ppu, rom, &mut tmp_frame);
            sprite_texture
                .update(
                    Some(sdl2::rect::Rect::new(
                        0,
                        0,
                        SPRITE_TABLE_DIMENSIONS.0,
                        SPRITE_TABLE_DIMENSIONS.0,
                    )),
                    &tmp_frame.data[0..tmp_frame.width * 3],
                    tmp_frame.width * 3,
                )
                .unwrap();
            tmp_frame.fill(ppu.get_color_from_current_system_palette(
                ppu.get_universal_background_color() as usize,
            ));
            render_oam_with_pos(ppu, rom, &mut tmp_frame);
            sprite_texture
                .update(
                    Some(sdl2::rect::Rect::new(
                        SPRITE_TABLE_DIMENSIONS.0 as i32,
                        0,
                        256,
                        240,
                    )),
                    &tmp_frame.data[0..tmp_frame.width * 3],
                    tmp_frame.width * 3,
                )
                .unwrap();
            front_end_state_rendering
                .borrow_mut()
                .sprite_canvas
                .copy(&sprite_texture, None, None)
                .unwrap();
            front_end_state_rendering
                .borrow_mut()
                .sprite_canvas
                .present();
        }

        if frame_counter.is_multiple_of(&10) {
            let palette = [
                ppu.read_palette_table(0),
                ppu.read_palette_table(1),
                ppu.read_palette_table(2),
                ppu.read_palette_table(3),
            ];
            if front_end_state_rendering.borrow().actions.show_tiles {
                let num_tiles = rom.chr_rom_len() / 16;
                for i in 0..num_tiles {
                    tmp_frame.render_tile((i % 32) * 8, (i / 32) * 8, &rom, 0, i, &palette);
                }
                let num_rows = num_tiles * 8 / tmp_frame.width;
                tile_texture
                    .update(
                        Some(sdl2::rect::Rect::new(0, 0, 256, num_rows as u32 * 8)),
                        &tmp_frame.data[0..tmp_frame.width * 3],
                        tmp_frame.width * 3,
                    )
                    .unwrap();
                front_end_state_rendering
                    .borrow_mut()
                    .tile_canvas
                    .copy(&tile_texture, None, None)
                    .unwrap();
                front_end_state_rendering.borrow_mut().tile_canvas.present();
            }
        }

        frame_counter += 1;
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

        if front_end_state.borrow().actions.show_tile_map {
            front_end_state
                .borrow_mut()
                .tile_map_canvas
                .window_mut()
                .show();
        } else {
            front_end_state
                .borrow_mut()
                .tile_map_canvas
                .window_mut()
                .hide();
        }

        if front_end_state.borrow().actions.show_tiles {
            front_end_state.borrow_mut().tile_canvas.window_mut().show();
        } else {
            front_end_state.borrow_mut().tile_canvas.window_mut().hide();
        }

        if front_end_state.borrow().actions.show_palette {
            front_end_state
                .borrow_mut()
                .palette_canvas
                .window_mut()
                .show();
        } else {
            front_end_state
                .borrow_mut()
                .palette_canvas
                .window_mut()
                .hide();
        }

        if front_end_state.borrow().actions.show_sprites {
            front_end_state
                .borrow_mut()
                .sprite_canvas
                .window_mut()
                .show();
        } else {
            front_end_state
                .borrow_mut()
                .sprite_canvas
                .window_mut()
                .hide();
        }

        //TODO: would be nicer to have a nes struct instead of doing this on the CPU
        if last_speed != front_end_state.borrow().actions.speed_multiplier {
            last_speed = front_end_state.borrow().actions.speed_multiplier;
            cpu.set_speed_multiplayer(last_speed);
        }

        if !pause {
            for _ in 0..1000 {
                cpu.step();
            }
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
