use itertools::Itertools;
use nes_rs::{
    bus::{ControllerCallback, GraphicsCallback},
    controller::{Controller, ControllerButtons, ControllerInput},
    ppu::{palette::SystemPalette, PPU},
    rendering::frame::{Frame, SCREEN_HEIGHT, SCREEN_WIDTH},
    ring_buffer::RingBuffer,
    rom::Rom,
    NES,
};
use sdl2::{
    controller::{Button, GameController},
    keyboard::Keycode,
    render::{BlendMode, WindowCanvas},
    AudioSubsystem, EventPump, GameControllerSubsystem, Sdl,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io::{self, Read},
    rc::Rc,
};

use crate::front_end::video::{
    create_window, creates_canvas_and_texture_creator, TextureCreators, Textures,
};

pub mod audio;
pub mod input;
pub mod video;

const PALETTE_VIEWER_DIMENSIONS: (u32, u32) = (4 * 8, 8 * 8);
const SPRITE_TABLE_DIMENSIONS: (u32, u32) = (8 * 8, 8 * 8);
const SPRITE_VIEW_DIMENSIONS: (u32, u32) = (SPRITE_TABLE_DIMENSIONS.0 + 256, 240);
const GRID_PIXEL_IN_NES_PIXEL: u32 = 2;
const BACKGROUND_COLOR: Option<(u8, u8, u8)> = Some((0x66, 0x66, 0x66));

//TODO: make history size configurable
pub const HISTORY_SIZE: usize = 1800;
pub type RewindBuffer = RingBuffer<(Frame, Vec<u8>), HISTORY_SIZE>;

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
    pub key_map_1: HashMap<Keycode, ControllerButtons>,
    pub key_map_2: HashMap<Keycode, ControllerButtons>,
    pub controller_map_1: HashMap<Button, ControllerButtons>,
    pub nes_controller_input: Vec<ControllerInput>,
    pub rewind_slot: usize,
}

impl FrontEndState {
    pub fn new(rom_name: &str, scaling: u32, integer_scaling: bool) -> (Self, TextureCreators) {
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

        let mut controller_map_1 = HashMap::new();
        controller_map_1.insert(Button::DPadDown, ControllerButtons::DOWN);
        controller_map_1.insert(Button::DPadUp, ControllerButtons::UP);
        controller_map_1.insert(Button::DPadRight, ControllerButtons::RIGHT);
        controller_map_1.insert(Button::DPadLeft, ControllerButtons::LEFT);
        controller_map_1.insert(Button::B, ControllerButtons::A);
        controller_map_1.insert(Button::A, ControllerButtons::B);
        controller_map_1.insert(Button::Start, ControllerButtons::SELECT);
        controller_map_1.insert(Button::Back, ControllerButtons::START);

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
                key_map_1,
                key_map_2,
                controller_map_1,
                nes_controller_input: Vec::new(),
                rewind_slot: 0,
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
    pub save_state: bool,
    pub load_state: bool,
    pub speed_multiplier: f64,
    pub rewind_mode: bool,
    pub rewind_move_left: bool,
    pub rewind_move_right: bool,
    pub rewind_load_slot: Option<usize>,
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
            save_state: false,
            load_state: false,
            speed_multiplier: 1f64,
            rewind_mode: false,
            rewind_move_left: false,
            rewind_move_right: false,
            rewind_load_slot: None,
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

pub fn create_callbacks<'a>(
    front_end_state: Rc<RefCell<FrontEndState>>,
    palette: SystemPalette,
    texture_creators: &'a TextureCreators,
    font_chr_rom: &'static [u8],
    rewind_buffer: Rc<RefCell<RewindBuffer>>,
) -> (impl GraphicsCallback<'a>, impl ControllerCallback<'a>) {
    let front_end_state_controller = front_end_state.clone();
    let front_end_state_rendering = front_end_state.clone();

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
            &rewind_buffer.borrow(),
        );
        // front_end_state_rendering.borrow_mut().actions.pause = true;

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

pub fn read_palette_table(path: &str) -> io::Result<SystemPalette> {
    let mut palette_file = File::open(path)?;
    let mut buffer = Vec::new();
    palette_file.read_to_end(&mut buffer)?;
    Ok(SystemPalette::from_raw(&buffer).unwrap())
}
