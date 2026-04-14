use std::{collections::HashMap, sync::LazyLock};

use nes_rs_core::controller::ControllerButtons;
use sdl2::{controller::Button, keyboard::Keycode};

use crate::front_end::input::UserInput;

/// Default key map for system/UI actions
pub static DEFAULT_SYSTEM_KEY_MAP: LazyLock<HashMap<Keycode, UserInput>> = LazyLock::new(|| {
    HashMap::from([
        (Keycode::Q, UserInput::Exit { window_id: 0 }),
        (Keycode::M, UserInput::TileMap),
        (Keycode::T, UserInput::TileView),
        (Keycode::O, UserInput::SpriteView),
        (Keycode::P, UserInput::PaletteView),
        (Keycode::F, UserInput::Fps),
        (Keycode::G, UserInput::ShowGrid),
        (Keycode::Escape, UserInput::Pause),
        (Keycode::E, UserInput::SaveState),
        (Keycode::R, UserInput::LoadSaveState),
        (Keycode::Z, UserInput::RewindView),
        (Keycode::Plus, UserInput::FastForward),
        (Keycode::Minus, UserInput::SlowDown),
        (Keycode::Right, UserInput::RewindRight),
        (Keycode::Left, UserInput::RewindLeft),
        (Keycode::Space, UserInput::RewindLoad),
        (Keycode::Y, UserInput::RecordInput),
        (Keycode::X, UserInput::ReplayInput),
        (Keycode::W, UserInput::TakeScreenShot),
    ])
});

/// Default key map for system/UI actions
pub static DEFAULT_SYSTEM_CONTROLLER_MAP: LazyLock<HashMap<Button, UserInput>> =
    LazyLock::new(|| {
        HashMap::from([
            (Button::RightStick, UserInput::FastForward),
            (Button::LeftStick, UserInput::SlowDown),
            (Button::LeftShoulder, UserInput::SaveState),
            (Button::RightShoulder, UserInput::LoadSaveState),
            (Button::Misc1, UserInput::RewindView),
            (Button::DPadRight, UserInput::RewindRight),
            (Button::DPadLeft, UserInput::RewindLeft),
            (Button::B, UserInput::RewindLoad),
        ])
    });

/// Default key map for controller 1
pub static DEFAULT_KEY_MAP_1: LazyLock<HashMap<Keycode, UserInput>> = LazyLock::new(|| {
    HashMap::from([
        (Keycode::Down, UserInput::Nes(ControllerButtons::DOWN)),
        (Keycode::Up, UserInput::Nes(ControllerButtons::UP)),
        (Keycode::Right, UserInput::Nes(ControllerButtons::RIGHT)),
        (Keycode::Left, UserInput::Nes(ControllerButtons::LEFT)),
        (Keycode::A, UserInput::Nes(ControllerButtons::A)),
        (Keycode::S, UserInput::Nes(ControllerButtons::B)),
        (Keycode::Space, UserInput::Nes(ControllerButtons::SELECT)),
        (Keycode::Return, UserInput::Nes(ControllerButtons::START)),
    ])
});

/// Default key map for controller 2
pub static DEFAULT_KEY_MAP_2: LazyLock<HashMap<Keycode, UserInput>> = LazyLock::new(|| {
    HashMap::from([
        (Keycode::J, UserInput::Nes(ControllerButtons::DOWN)),
        (Keycode::K, UserInput::Nes(ControllerButtons::UP)),
        (Keycode::L, UserInput::Nes(ControllerButtons::RIGHT)),
        (Keycode::H, UserInput::Nes(ControllerButtons::LEFT)),
        (Keycode::U, UserInput::Nes(ControllerButtons::A)),
        (Keycode::I, UserInput::Nes(ControllerButtons::B)),
        (Keycode::B, UserInput::Nes(ControllerButtons::SELECT)),
        (Keycode::N, UserInput::Nes(ControllerButtons::START)),
    ])
});

/// Default controller map for controller 1
pub static DEFAULT_CONTROLLER_MAP_1: LazyLock<HashMap<Button, UserInput>> = LazyLock::new(|| {
    HashMap::from([
        (Button::DPadDown, UserInput::Nes(ControllerButtons::DOWN)),
        (Button::DPadUp, UserInput::Nes(ControllerButtons::UP)),
        (Button::DPadRight, UserInput::Nes(ControllerButtons::RIGHT)),
        (Button::DPadLeft, UserInput::Nes(ControllerButtons::LEFT)),
        (Button::B, UserInput::Nes(ControllerButtons::A)),
        (Button::A, UserInput::Nes(ControllerButtons::B)),
        (Button::Start, UserInput::Nes(ControllerButtons::SELECT)),
        (Button::Back, UserInput::Nes(ControllerButtons::START)),
    ])
});

/// Default controller map for controller 2
pub static DEFAULT_CONTROLLER_MAP_2: LazyLock<HashMap<Button, UserInput>> = LazyLock::new(|| {
    HashMap::from([
        (Button::DPadDown, UserInput::Nes(ControllerButtons::DOWN)),
        (Button::DPadUp, UserInput::Nes(ControllerButtons::UP)),
        (Button::DPadRight, UserInput::Nes(ControllerButtons::RIGHT)),
        (Button::DPadLeft, UserInput::Nes(ControllerButtons::LEFT)),
        (Button::B, UserInput::Nes(ControllerButtons::A)),
        (Button::A, UserInput::Nes(ControllerButtons::B)),
        (Button::Start, UserInput::Nes(ControllerButtons::SELECT)),
        (Button::Back, UserInput::Nes(ControllerButtons::START)),
    ])
});
