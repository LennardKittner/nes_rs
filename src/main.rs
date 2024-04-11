use std::collections::HashMap;
use std::{env, io};
use std::fs::File;
use std::io::Read;
use itertools::Itertools;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use nes_rs::bus::Bus;
use nes_rs::controller::{Controller, ControllerButtons};
use nes_rs::cpu::CPU;
use nes_rs::rendering::frame::Frame;
use nes_rs::ppu::PPU;
use nes_rs::ppu::pallet::SystemPallet;
use nes_rs::rom::Rom;

fn main() {
    let args = env::args().collect_vec();
    if args.len() < 2 {
        println!("Please provide the path to a NES rom");
        return;
    }
    let rom_path = &args[1];
    let mut palette_path = None;
    if args.len() >= 3 {
        palette_path = Some(&args[2]);
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(&format!("NESrs -- {rom_path}"), (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let bytes: Vec<u8> = std::fs::read(rom_path).unwrap();
    let rom = Rom::new(&bytes).unwrap();

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

    let poll_controller_input = move |controller_1: &mut Controller, controller_2: &mut Controller| {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    if let Some(key) = key_map_1.get(&keycode) {
                        controller_1.set_button_state(true, *key);
                    } else if let Some(key) = key_map_2.get(&keycode) {
                        controller_2.set_button_state(true, *key);
                    }
                }
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    if let Some(key) = key_map_1.get(&keycode) {
                        controller_1.set_button_state(false, *key);
                    } else if let Some(key) = key_map_2.get(&keycode) {
                        controller_2.set_button_state(false, *key);
                    }
                }
                _ => { /* do nothing */ }
            }
        }
    };

    let palette = if let Some(path) = palette_path {
        read_palette_table(path).unwrap_or(SystemPallet::new())
    } else {
        SystemPallet::new()
    };

    let bus = Bus::new(rom, palette,
        move |_: &PPU, frame: &Frame | {
            texture.update(None, &frame.data, 256 * 3).unwrap();

            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
            }, poll_controller_input);

    let mut cpu = CPU::new_with_bus(bus);
    cpu.reset();
    cpu.run();
}

fn read_palette_table(path: &str) -> io::Result<SystemPallet> {
    let mut palette_file = File::open(path)?;
    let mut buffer = Vec::new();
    palette_file.read_to_end(&mut buffer)?;
    Ok(SystemPallet::from_raw(&buffer).unwrap())
}