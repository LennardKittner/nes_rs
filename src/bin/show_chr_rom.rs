use itertools::Itertools;
use nes_rs::rendering::frame::Frame;
use nes_rs::rom::Rom;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::env;

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
        .window("Tile viewer", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
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

    let mut frame = Frame::default();
    for i in 0..(rom.chr_rom_len() / 16) {
        frame.render_tile(
            (i % 32) * 8,
            (i / 32) * 8,
            &rom,
            0,
            i,
            &[0x0F, 0x30, 0x21, 0x0F],
        );
    }

    texture.update(None, &frame.data, 256 * 3).unwrap();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                _ => { /* do nothing */ }
            }
        }
    }
}
