use nes_rs::controller::ControllerInput;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
};

use crate::front_end::FrontEndState;

//TODO: use keymap for other controls too
// allow setting of keymap
// support multiple physical controller
// use controller for rewind

pub fn handle_user_input(front_end: &mut FrontEndState) {
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
                    front_end.rewind_slot = 0;
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
                front_end.rewind_slot = 0;
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
            Event::ControllerButtonDown { button, .. } => {
                if let Some(&key) = front_end.controller_map_1.get(&button) {
                    front_end
                        .nes_controller_input
                        .push(ControllerInput::Controller1(true, key));
                }
            }
            Event::ControllerButtonUp { button, .. } => {
                if let Some(&key) = front_end.controller_map_1.get(&button) {
                    front_end
                        .nes_controller_input
                        .push(ControllerInput::Controller1(false, key));
                }
            }
            Event::KeyDown {
                keycode: Some(keycode),
                ..
            } => {
                if front_end.actions.rewind_mode && keycode == Keycode::Left {
                    front_end.actions.rewind_move_left = true;
                    continue;
                } else if front_end.actions.rewind_mode && keycode == Keycode::Right {
                    front_end.actions.rewind_move_right = true;
                    continue;
                } else if front_end.actions.rewind_mode && keycode == Keycode::Space {
                    front_end.actions.rewind_load_slot = Some(front_end.rewind_slot);
                    continue;
                }
                println!("{}", keycode);
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
