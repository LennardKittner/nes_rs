use std::collections::HashMap;
use std::env;
use itertools::Itertools;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use nes_rs::bus::Bus;
use nes_rs::controller::{Controller, ControllerButtons};
use nes_rs::cpu::CPU;
use nes_rs::frame::Frame;
use nes_rs::ppu::PPU;
use nes_rs::render::render;
use nes_rs::rom::Rom;

fn main() {
    let args = env::args().collect_vec();
    if args.len() < 2 {
        println!("Please provide the path to a NES rom");
        return;
    }
    let path = &args[1];

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(&format!("NESrs -- {path}"), (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
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

    let bytes: Vec<u8> = std::fs::read(path).unwrap();
    let rom = Rom::new(&bytes).unwrap();
    let mut frame = Frame::new();

    //TODO: second controller
    //TODO: input buffering
    let mut key_map = HashMap::new();
    key_map.insert(Keycode::Down, ControllerButtons::DOWN);
    key_map.insert(Keycode::Up, ControllerButtons::UP);
    key_map.insert(Keycode::Right, ControllerButtons::RIGHT);
    key_map.insert(Keycode::Left, ControllerButtons::LEFT);
    key_map.insert(Keycode::A, ControllerButtons::A);
    key_map.insert(Keycode::S, ControllerButtons::B);
    key_map.insert(Keycode::Space, ControllerButtons::SELECT);
    key_map.insert(Keycode::Return, ControllerButtons::START);

    let bus = Bus::new(rom, move |ppu: &PPU, controller_1: &mut Controller, _: &mut Controller | {
        render(ppu, &mut frame);
        texture.update(None, &frame.data, 256 * 3).unwrap();

        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    if let Some(key) = key_map.get(&keycode) {
                        controller_1.set_button_state(true, *key);
                    }
                }
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    if let Some(key) = key_map.get(&keycode) {
                        controller_1.set_button_state(false, *key);
                    }
                }
                _ => { /* do nothing */ }
            }
        }
    });

    let mut cpu = CPU::new_with_bus(bus);
    cpu.reset();
    cpu.run();
}
