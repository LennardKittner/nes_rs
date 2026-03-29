use itertools::Itertools;
use nes_rs::{
    NES,
    bus::{ControllerCallback, GraphicsCallback},
    controller::{Controller, ControllerInput},
    ppu::{PPU, palette::SystemPalette},
    rendering::frame::{Frame, SCREEN_HEIGHT, SCREEN_WIDTH},
    ring_buffer::RingBuffer,
    rom::Rom,
};
use sdl2::{
    AudioSubsystem, EventPump, GameControllerSubsystem, Sdl,
    controller::{Button, GameController},
    keyboard::Keycode,
    render::{BlendMode, WindowCanvas},
};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, File},
    io::{self, Read},
    rc::Rc,
};

use crate::front_end::{
    default_key_maps::{
        DEFAULT_CONTROLLER_MAP_1, DEFAULT_CONTROLLER_MAP_2, DEFAULT_KEY_MAP_1, DEFAULT_KEY_MAP_2,
        DEFAULT_SYSTEM_CONTROLLER_MAP, DEFAULT_SYSTEM_KEY_MAP,
    },
    input::{ButtonPress, UserInput},
    video::{TextureCreators, Textures, create_window, creates_canvas_and_texture_creator},
};

pub mod audio;
pub mod default_key_maps;
pub mod input;
pub mod video;

const PALETTE_VIEWER_DIMENSIONS: (u32, u32) = (4 * 8, 8 * 8);
const SPRITE_TABLE_DIMENSIONS: (u32, u32) = (8 * 8, 8 * 8);
const SPRITE_VIEW_DIMENSIONS: (u32, u32) = (SPRITE_TABLE_DIMENSIONS.0 + 256, 240);
const GRID_PIXEL_IN_NES_PIXEL: u32 = 2;
const BACKGROUND_COLOR: Option<(u8, u8, u8)> = Some((0x66, 0x66, 0x66));

//TODO: allow setting of keymap
//TODO: make history size configurable
pub const HISTORY_SIZE: usize = 1800;
pub type RewindBuffer = RingBuffer<(Frame, Vec<u8>), HISTORY_SIZE>;

/// Contains state related to the front end e.g. SDL2 sound, video, and input subsystems
#[allow(dead_code)]
pub struct FrontEndState {
    pub sdl_context: Sdl,
    pub actions: Actions,
    pub controller_subsystem: GameControllerSubsystem,
    pub controllers: Vec<GameController>,
    pub audio_subsystem: AudioSubsystem,
    pub tile_map_canvas: WindowCanvas,
    pub main_canvas: WindowCanvas,
    pub tile_canvas: WindowCanvas,
    pub palette_canvas: WindowCanvas,
    pub sprite_canvas: WindowCanvas,
    pub event_pump: EventPump,
    pub key_map_1: HashMap<Keycode, UserInput>,
    pub key_map_2: HashMap<Keycode, UserInput>,
    pub system_key_map: HashMap<Keycode, UserInput>,
    pub controller_map_1: HashMap<Button, UserInput>,
    pub controller_map_2: HashMap<Button, UserInput>,
    pub system_controller_map: HashMap<Button, UserInput>,
    pub nes_controller_input: Vec<ControllerInput>,
    pub rewind_slot: usize,
    pub input_replay_buffer: Vec<ButtonPress>,
    pub input_recording_buffer: Vec<ButtonPress>,
    pub rewind_buffer: RewindBuffer,
    pub save_state_path: String,
    pub input_recording_path: String,
}

impl FrontEndState {
    /// create a new FrontEndState
    pub fn new(
        rom_name: &str,
        scaling: u32,
        integer_scaling: bool,
        save_state_path: &str,
        input_recording_path: &str,
    ) -> (Self, TextureCreators) {
        let sdl_context = sdl2::init().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let controller_subsystem = sdl_context.game_controller().unwrap();
        let num_controller = controller_subsystem.num_joysticks().unwrap();
        let controllers = (0..num_controller)
            .filter_map(|id| {
                if controller_subsystem.is_game_controller(id) {
                    controller_subsystem.open(id).ok()
                } else {
                    None
                }
            })
            .collect_vec();

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

        let (tile_map_canvas, tile_map_creator) = creates_canvas_and_texture_creator(
            window_tile_map,
            SCREEN_WIDTH as u32 * 2 * 2,
            SCREEN_HEIGHT as u32 * 2 * 2,
            integer_scaling,
        );

        let (tile_canvas, tile_creator) = creates_canvas_and_texture_creator(
            window_tile,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
            integer_scaling,
        );

        let (mut main_canvas, main_creator) = creates_canvas_and_texture_creator(
            window,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
            integer_scaling,
        );
        main_canvas.set_blend_mode(BlendMode::Blend);

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

        (
            Self {
                sdl_context,
                actions: Actions::new(),
                audio_subsystem,
                controller_subsystem,
                controllers,
                tile_map_canvas,
                main_canvas,
                tile_canvas,
                palette_canvas,
                sprite_canvas,
                event_pump,
                key_map_1: DEFAULT_KEY_MAP_1.clone(),
                key_map_2: DEFAULT_KEY_MAP_2.clone(),
                controller_map_1: DEFAULT_CONTROLLER_MAP_1.clone(),
                controller_map_2: DEFAULT_CONTROLLER_MAP_2.clone(),
                system_controller_map: DEFAULT_SYSTEM_CONTROLLER_MAP.clone(),
                system_key_map: DEFAULT_SYSTEM_KEY_MAP.clone(),
                nes_controller_input: Vec::new(),
                rewind_slot: 0,
                input_replay_buffer: Vec::new(),
                input_recording_buffer: Vec::new(),
                rewind_buffer: RewindBuffer::new(),
                save_state_path: save_state_path.to_string(),
                input_recording_path: input_recording_path.to_string(),
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

    /// Hide / Unhide active windows
    pub fn show_active_windows(&mut self) {
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

pub struct Actions {
    pub should_quit: bool,
    pub pause: bool,
    pub show_tile_map: bool,
    pub show_tiles: bool,
    pub show_palette: bool,
    pub show_sprites: bool,
    pub show_fps: bool,
    pub show_grid: bool,
    pub speed_multiplier: f64,
    pub rewind_mode: bool,
    pub record_input: bool,
    pub replay_input: bool,
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
            show_grid: false,
            speed_multiplier: 1f64,
            rewind_mode: false,
            record_input: false,
            replay_input: false,
        }
    }
}

//TODO: maybe use bitcode with encode decode bitcode + serde is slower
pub fn create_save_state_bin(nes: &NES) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_stdvec(&nes)
}

#[allow(dead_code)]
pub fn create_save_state_json(nes: &NES) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&nes)
}

pub fn resume_from_save_state_bin<'a>(nes: NES<'a>, data: &[u8]) -> Option<NES<'a>> {
    nes.replace_state(postcard::from_bytes(data).ok()?)
}

#[allow(dead_code)]
pub fn resume_from_save_state_json<'a>(nes: NES<'a>, data: &str) -> Option<NES<'a>> {
    nes.replace_state(serde_json::from_str(data).ok()?)
}

/// Create graphics and input callbacks
pub fn create_callbacks<'a>(
    front_end_state: Rc<RefCell<FrontEndState>>,
    palette: SystemPalette,
    texture_creators: &'a TextureCreators,
    font_chr_rom: &'static [u8],
) -> (impl GraphicsCallback<'a>, impl ControllerCallback<'a>) {
    let front_end_state_controller = front_end_state.clone();
    let front_end_state_rendering = front_end_state.clone();

    let handle_controller_input =
        move |controller_1: &mut Controller, controller_2: &mut Controller, cycle: u64| {
            let mut register_input = |button| match button {
                ControllerInput::Controller1(pressed, key) => {
                    controller_1.set_button_state(pressed, key)
                }
                ControllerInput::Controller2(pressed, key) => {
                    controller_2.set_button_state(pressed, key)
                }
            };
            let mut front_end = front_end_state_controller.borrow_mut();
            let inputs = std::mem::take(&mut front_end.nes_controller_input);
            let replay = front_end.actions.replay_input;
            if let Some(input) = front_end
                .input_replay_buffer
                .pop_if(|input| replay && input.cycle <= cycle)
            {
                register_input(input.button);
                if front_end.input_replay_buffer.is_empty() {
                    front_end.actions.replay_input = false;
                }
            }

            if !replay {
                for button in inputs {
                    if front_end.actions.record_input {
                        front_end
                            .input_recording_buffer
                            .push(ButtonPress { button, cycle });
                    }
                    register_input(button);
                }
            }
        };

    let mut textures = Textures::new(
        &mut front_end_state.borrow_mut(),
        texture_creators,
        palette.clone(),
    );

    let render_frame = move |ppu: &PPU, frame: &Frame, fps: u32, rom: &Rom| {
        textures.update_textures(
            frame,
            fps,
            &mut front_end_state_rendering.borrow_mut(),
            ppu,
            rom,
            font_chr_rom,
            &palette,
        );

        //TODO: add screenshot function and maybe video recoding
        // image::save_buffer(
        //     format!("./{rom_name}.png"),
        //     &frame.data,
        //     SCREEN_WIDTH as u32,
        //     SCREEN_HEIGHT as u32,
        //     image::ColorType::Rgb8,
        // )
        // .unwrap();
    };
    (render_frame, handle_controller_input)
}

/// read the pallet table from a file
pub fn read_palette_table(path: &str) -> io::Result<SystemPalette> {
    let mut palette_file = File::open(path)?;
    let mut buffer = Vec::new();
    palette_file.read_to_end(&mut buffer)?;
    Ok(SystemPalette::from_raw(&buffer).unwrap())
}

/// handles user input
pub fn handle_user_input<'a>(
    user_input: &UserInput,
    front_end: &mut FrontEndState,
    mut nes: NES<'a>,
) -> NES<'a> {
    match user_input {
        UserInput::TileMap => {
            front_end.actions.show_tile_map ^= true;
        }
        UserInput::TileView => front_end.actions.show_tiles ^= true,
        UserInput::SpriteView => front_end.actions.show_sprites ^= true,
        UserInput::PaletteView => front_end.actions.show_palette ^= true,
        UserInput::Fps => front_end.actions.show_fps ^= true,
        UserInput::ShowGrid => front_end.actions.show_grid ^= true,
        UserInput::RecordInput => {
            if !front_end.actions.replay_input {
                if front_end.actions.record_input {
                    let json = serde_json::ser::to_string_pretty(&std::mem::take(
                        &mut front_end.input_recording_buffer,
                    ))
                    .unwrap();
                    if let Err(e) = fs::write(&front_end.input_recording_path, json) {
                        eprint!("Failed to save input: {e}");
                    }
                }
                front_end.actions.record_input ^= true
            }
        }
        UserInput::ReplayInput => {
            if !front_end.actions.record_input {
                if !front_end.actions.replay_input {
                    let json = fs::read_to_string(&front_end.input_recording_path).unwrap();
                    front_end.input_replay_buffer = serde_json::de::from_str(&json).unwrap();
                    front_end.input_replay_buffer.reverse();
                }
                front_end.actions.replay_input ^= true;
            }
        }
        UserInput::Pause => {
            front_end.actions.pause ^= true;
            if front_end.actions.rewind_mode && !front_end.actions.pause {
                front_end.actions.rewind_mode = false;
                front_end.rewind_slot = 0;
            }
        }
        UserInput::SaveState => save_state_to_disk(&mut nes, &front_end.save_state_path),
        UserInput::LoadSaveState => {
            nes = load_save_state_from_disk(nes, &front_end.save_state_path)
        }
        UserInput::RewindView => {
            front_end.actions.rewind_mode ^= true;
            front_end.actions.pause = front_end.actions.rewind_mode;
            front_end.rewind_slot = 0;
        }
        UserInput::FastForward => {
            front_end.actions.speed_multiplier += 1f64;
            front_end.actions.speed_multiplier =
                front_end.actions.speed_multiplier.clamp(1f64, 50f64);
            nes.set_speed_multiplayer(front_end.actions.speed_multiplier)
        }
        UserInput::SlowDown => {
            front_end.actions.speed_multiplier -= 1f64;
            front_end.actions.speed_multiplier =
                front_end.actions.speed_multiplier.clamp(1f64, 50f64);
            nes.set_speed_multiplayer(front_end.actions.speed_multiplier)
        }
        UserInput::Exit { window_id } => {
            if front_end.main_canvas.window().id() == *window_id {
                front_end.actions.should_quit = true;
            } else if front_end.tile_canvas.window().id() == *window_id {
                front_end.actions.show_tiles = false;
            } else if front_end.tile_map_canvas.window().id() == *window_id {
                front_end.actions.show_tile_map = false;
            } else if front_end.sprite_canvas.window().id() == *window_id {
                front_end.actions.show_sprites = false;
            } else if front_end.palette_canvas.window().id() == *window_id {
                front_end.actions.show_palette = false;
            } else {
                front_end.actions.should_quit = true;
            }
        }
        UserInput::RewindLeft => {
            front_end.rewind_slot =
                (front_end.rewind_slot + 1).clamp(0, front_end.rewind_buffer.writer_head - 1);
        }
        UserInput::RewindRight => {
            front_end.rewind_slot = front_end.rewind_slot.saturating_sub(1);
        }
        UserInput::RewindLoad => {
            if front_end.actions.rewind_mode {
                nes = continue_from_rewind_slot(nes, front_end);
            }
        }
        UserInput::NesFull(controller_input) => {
            front_end.nes_controller_input.push(*controller_input)
        }
        UserInput::Nes(_) => {
            panic!("This variant should only be used inside the key map")
        }
    }
    nes
}

/// loads the save state from disk a nd applies it
fn load_save_state_from_disk<'a>(nes: NES<'a>, path: &str) -> NES<'a> {
    let save = match fs::read(path) {
        Ok(save) => save,
        Err(err) => {
            eprintln!("Failed to load save state: {err}");
            Vec::new()
        }
    };
    if !save.is_empty() {
        if let Some(new_nes) = resume_from_save_state_bin(nes, &save) {
            return new_nes;
        } else {
            panic!("Failed to resume from save state.")
        }
    }
    nes
}

/// stores the save state to disk
fn save_state_to_disk(nes: &mut NES, path: &str) {
    let save = match create_save_state_bin(nes) {
        Ok(save) => save,
        Err(err) => {
            eprintln!("Failed to create save state: {err}");
            Vec::new()
        }
    };
    if !save.is_empty()
        && let Err(err) = fs::write(path, save)
    {
        eprintln!("Failed to write save state: {err}");
    }
}

/// load the save from the rewind slot specified in front_end_state
fn continue_from_rewind_slot<'a>(nes: NES<'a>, front_end_state: &mut FrontEndState) -> NES<'a> {
    let rewind_slot = front_end_state.rewind_slot;
    let rewind_slot = front_end_state
        .rewind_buffer
        .writer_head
        .saturating_sub(rewind_slot + 1);
    let (_, state) = front_end_state.rewind_buffer.get(rewind_slot).unwrap();
    if let Some(new_nes) = resume_from_save_state_bin(nes, &state) {
        front_end_state.rewind_slot = 0;
        front_end_state.actions.rewind_mode = false;
        front_end_state.actions.pause = false;
        new_nes
    } else {
        panic!("Failed to resume from save state.")
    }
}
