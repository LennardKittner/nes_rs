use std::collections::HashMap;

use nes_rs::controller::{ControllerButtons, ControllerInput};
use sdl2::{
    controller::Button,
    event::{Event, WindowEvent},
    keyboard::Keycode,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
/// Represents a button press at a specific point in time
pub struct ButtonPress {
    /// the button
    pub button: ControllerInput,
    /// the cycle on which the input happened
    pub cycle: u64,
}

#[derive(Debug, Serialize, Deserialize)]
/// Represents a sequence of inputs with the cycles offsets at which the recording / replay started
pub struct InputBuffer {
    /// when the recording / replay started
    pub cycle_offset: u64,
    /// input sequence
    pub input: Vec<ButtonPress>,
}

impl InputBuffer {
    /// create an empty buffer
    pub fn new() -> Self {
        InputBuffer {
            cycle_offset: 0,
            input: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
/// Meta representation of user Input
pub enum UserInput {
    TileMap,
    TileView,
    SpriteView,
    PaletteView,
    Fps,
    ShowGrid,
    RecordInput,
    ReplayInput,
    Pause,
    SaveState,
    LoadSaveState,
    RewindView,
    FastForward,
    SlowDown,
    Exit { window_id: u32 },
    RewindLeft,
    RewindRight,
    RewindLoad,
    Nes(ControllerButtons),
    NesFull(ControllerInput),
}

impl UserInput {
    fn convert_nes_controller_input(self, controller_1: bool, down: bool) -> UserInput {
        match self {
            UserInput::Nes(button) => {
                if controller_1 {
                    UserInput::NesFull(ControllerInput::Controller1(down, button))
                } else {
                    UserInput::NesFull(ControllerInput::Controller2(down, button))
                }
            }
            input => input,
        }
    }
}

/// transforms user input from keyboard or other sources to UserInput based on the keymaps
pub fn parse_user_input(
    key_map_1: &HashMap<Keycode, UserInput>,
    key_map_2: &HashMap<Keycode, UserInput>,
    key_map_system: &HashMap<Keycode, UserInput>,
    controller_map_1: &HashMap<Button, UserInput>,
    controller_map_2: &HashMap<Button, UserInput>,
    controller_map_system: &HashMap<Button, UserInput>,
    event: Event,
) -> Vec<UserInput> {
    match event {
        Event::Quit { .. } => {
            vec![UserInput::Exit { window_id: 0 }]
        }
        Event::Window {
            timestamp: _,
            window_id,
            win_event: WindowEvent::Close,
        } => {
            vec![UserInput::Exit { window_id }]
        }
        Event::ControllerButtonDown {
            button, which: 0, ..
        } => controller_map_system
            .get(&button)
            .cloned()
            .into_iter()
            .chain(controller_map_1.get(&button).cloned())
            .map(|input| input.convert_nes_controller_input(true, true))
            .collect(),
        Event::ControllerButtonUp {
            button, which: 0, ..
        } => controller_map_1
            .get(&button)
            .cloned()
            .into_iter()
            .map(|input| input.convert_nes_controller_input(true, false))
            .collect(),
        Event::ControllerButtonDown {
            button, which: 1, ..
        } => controller_map_2
            .get(&button)
            .cloned()
            .into_iter()
            .map(|input| input.convert_nes_controller_input(false, true))
            .collect(),
        Event::ControllerButtonUp {
            button, which: 1, ..
        } => controller_map_2
            .get(&button)
            .cloned()
            .into_iter()
            .map(|input| input.convert_nes_controller_input(false, false))
            .collect(),
        Event::KeyDown {
            keycode: Some(keycode),
            ..
        } => {
            let key_1 = key_map_system
                .get(&keycode)
                .cloned()
                .into_iter()
                .chain(key_map_1.get(&keycode).cloned())
                .map(|input| input.convert_nes_controller_input(true, true));

            key_map_2
                .get(&keycode)
                .cloned()
                .into_iter()
                .map(|input| input.convert_nes_controller_input(false, true))
                .chain(key_1)
                .collect()
        }
        Event::KeyUp {
            keycode: Some(keycode),
            ..
        } => {
            let key_1 = key_map_1
                .get(&keycode)
                .cloned()
                .into_iter()
                .map(|input| input.convert_nes_controller_input(true, false));

            key_map_2
                .get(&keycode)
                .cloned()
                .into_iter()
                .map(|input| input.convert_nes_controller_input(false, false))
                .chain(key_1)
                .collect()
        }
        _ => vec![],
    }
}
