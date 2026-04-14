use clap::Parser;
use itertools::Itertools;
use nes_rs_core::{NES, ppu::palette::SystemPalette, rom::Rom};
use std::{cell::RefCell, path::Path, rc::Rc, thread, time::Duration};

use crate::front_end::{
    FrontEndState, audio::AudioDeviceWrapper, create_callbacks, create_save_state_bin,
    handle_user_input, input::parse_user_input, read_palette_table,
};

mod front_end;

const FONT_NUMBERS_OFFSET: usize = 16;
const FONT_LETTERS_OFFSET: usize = 33;
const FONT_CHR_ROM: &[u8; 1536] = include_bytes!("../om_thick_plain_nes.chr");

//TODO: when loading save state load preview image to avoid black frame
//TODO: make most stuff pub(crate) instead of pub

//TODO: new input abstraction and recording also add recoding as input flag
// maybe only store with offset from recording start
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
    let recording_path = Path::new(&rom_path).with_extension("wav");
    let save_state_path = Path::new(&rom_path).with_extension("save_state");
    let input_recording_path = Path::new(&rom_path).with_extension("key.json");
    let screenshot_path = Path::new(&rom_path).with_extension("png");
    let palette_path = args.palette_path;

    let (front_end_state, texture_creators) = FrontEndState::new(
        rom_name,
        args.scaling,
        args.enable_integer_scaling,
        save_state_path.to_str().unwrap_or("unnamed.save_state"),
        input_recording_path.to_str().unwrap_or("unnamed.key.json"),
        screenshot_path.to_str().unwrap_or("unnamed.png"),
    );
    let front_end_state = Rc::new(RefCell::new(front_end_state));

    let rom = Rom::load_from_disk(&rom_path).unwrap();

    let palette = if let Some(path) = palette_path {
        read_palette_table(&path).unwrap_or_default()
    } else {
        SystemPalette::new()
    };

    let (render_frame, handle_controller_input) = create_callbacks(
        front_end_state.clone(),
        palette.clone(),
        &texture_creators,
        FONT_CHR_ROM,
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
            recording_path.to_str().unwrap(),
            audio_buffer.clone(),
        )
    } else {
        AudioDeviceWrapper::new(&front_end_state.borrow(), audio_buffer.clone())
    };

    audio_device_wrapper.audio_device.resume();
    let mut next_save_on_frame = 0;
    while !front_end_state.borrow().actions.should_quit {
        // Limit borrow of front_end_state
        {
            let mut front_end_state = front_end_state.borrow_mut();
            while let Some(event) = front_end_state.event_pump.poll_event() {
                for input in parse_user_input(
                    &front_end_state.key_map_1,
                    &front_end_state.key_map_2,
                    &front_end_state.system_key_map,
                    &front_end_state.controller_map_1,
                    &front_end_state.controller_map_2,
                    &front_end_state.system_controller_map,
                    event,
                ) {
                    nes = handle_user_input(&input, &mut front_end_state, nes);
                }
            }

            front_end_state.show_active_windows();

            if next_save_on_frame <= nes.get_frame_counter() {
                next_save_on_frame = nes.get_frame_counter() + 10;
                front_end_state.rewind_buffer.push((
                    nes.get_current_frame(),
                    create_save_state_bin(&nes).unwrap(),
                ));
            } else if (next_save_on_frame - nes.get_frame_counter()) > 10 {
                // a save state was loaded resetting the frame counter
                next_save_on_frame = nes.get_frame_counter() + 10;
            }
        }

        if !front_end_state.borrow().actions.pause {
            for _ in 0..1000 {
                nes.step(); // step may call callbacks accessing front_end_state so we have to stop
                // borrowing here
            }
        } else {
            thread::sleep(Duration::from_millis(16)); // Roughly 60FPS avoids wasting resources
            // when the emulation is paused
            nes.manual_re_render(); // without this windows such as the tile map would only show
            // once the emulation gets resumed
        }
    }
}
