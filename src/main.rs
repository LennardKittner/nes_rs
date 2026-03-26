use clap::Parser;
use itertools::Itertools;
use nes_rs::{ppu::palette::SystemPalette, rom::Rom, NES};
use std::{
    cell::RefCell,
    fs::{self},
    path::Path,
    rc::Rc,
    thread,
    time::Duration,
};

use crate::front_end::{
    audio::AudioDeviceWrapper, create_callbacks, create_save_state_bin, input::handle_user_input,
    read_palette_table, resume_from_save_state_bin, FrontEndState, RewindBuffer,
};

mod front_end;

const FONT_NUMBERS_OFFSET: usize = 16;
const FONT_LETTERS_OFFSET: usize = 33;
const FONT_CHR_ROM: &[u8; 1536] = include_bytes!("../om_thick_plain_nes.chr");

//TODO:
//xbox controller?
//Fix tests

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
    let save_state_path = Path::new(&rom_path).with_extension("save_state");
    let recording_path = Path::new(&rom_path).with_extension("wav");
    let palette_path = args.palette_path;

    let (front_end_state, texture_creators) =
        FrontEndState::new(rom_name, args.scaling, args.enable_integer_scaling);
    let front_end_state = Rc::new(RefCell::new(front_end_state));

    let rewind_buffer = Rc::new(RefCell::new(RewindBuffer::new()));

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
            recording_path.to_str().unwrap(),
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
}
