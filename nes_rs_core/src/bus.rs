use crate::apu::APU;
use crate::controller::Controller;
use crate::ppu::palette::SystemPalette;
use crate::ppu::{PPU, PRE_RENDER_SCNALINE, VBLANK_START};
use crate::rendering::{frame::Frame, scanline::Scanline};
use crate::ring_buffer::RingBuffer;
use crate::rolling_avg::RollingAvg;
use crate::rom::{Rom, RomState};
use derivative::Derivative;
use num::traits::Inv;
use serde::{Deserialize, Serialize};
use serde_with::Bytes;
use serde_with::serde_as;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const FRAME_DURATION: Duration = Duration::from_nanos(16666667);

pub const AUDIO_BUFFER_SIZE: usize = 44100;
pub type AudioBuffer = Arc<Mutex<RingBuffer<f32, AUDIO_BUFFER_SIZE>>>;
pub trait ControllerCallback<'a>: FnMut(&mut Controller, &mut Controller, u64) + 'a {}
impl<'a, F: FnMut(&mut Controller, &mut Controller, u64) + 'a> ControllerCallback<'a> for F {}

pub trait GraphicsCallback<'a>: FnMut(&PPU, &Frame, u32, &Rom) + 'a {}
impl<'a, F: FnMut(&PPU, &Frame, u32, &Rom) + 'a> GraphicsCallback<'a> for F {}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Bus<'a> {
    cpu_vram: [u8; 2048],
    pub rom: Rom,
    pub ppu: PPU,
    apu: Option<APU>,
    pub frame: Frame,
    scanline_buffers: [Scanline; 2],
    current_scanline_buffer: usize,
    last_scanline: i32,
    cycles: u64,
    open_bus: u8,
    #[derivative(Debug = "ignore")]
    pub graphics_callback: Box<dyn GraphicsCallback<'a>>,
    #[derivative(Debug = "ignore")]
    pub controller_callback: Box<dyn ControllerCallback<'a>>,
    pub controller_1: Controller,
    pub controller_2: Controller,
    last_frame: Instant,
    rendering_overhead: RollingAvg<u64>,
    last_60_frames: Instant,
    current_fps: u32,
    pub frame_counter: u64,
    desired_frame_duration: Duration,
    pub audio_ring_buffer: AudioBuffer, // 1s of audio
    pub system_palette: SystemPalette,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct BusState {
    #[serde_as(as = "Bytes")]
    cpu_vram: [u8; 2048],
    rom: RomState,
    ppu: PPU,
    apu: Option<APU>,
    last_scanline: i32,
    cycles: u64,
    open_bus: u8,
    frame_counter: u64,
}

impl BusState {
    pub fn new(bus: &Bus) -> BusState {
        BusState {
            cpu_vram: bus.cpu_vram,
            rom: RomState::new(&bus.rom),
            ppu: bus.ppu.clone(),
            apu: bus.apu.clone(),
            last_scanline: bus.last_scanline,
            cycles: bus.cycles,
            open_bus: bus.open_bus,
            frame_counter: bus.frame_counter,
        }
    }

    pub fn get_rom_hash(&self) -> u64 {
        self.rom.get_rom_hash()
    }
}

impl<'a> Bus<'a> {
    pub fn new(
        rom: Rom,
        system_palette: SystemPalette,
        speed_multiplier: f64,
        graphics_callback: impl GraphicsCallback<'a>,
        controller_callback: impl ControllerCallback<'a>,
    ) -> Bus<'a> {
        let ppu = PPU::new();
        let mut speed_multiplier = speed_multiplier;
        if speed_multiplier <= 0f64 {
            speed_multiplier = f64::INFINITY;
        }
        Bus {
            cpu_vram: [0; 2048],
            rom,
            cycles: 0,
            open_bus: 0,
            ppu,
            apu: Some(APU::new(speed_multiplier)),
            frame: Frame::default(),
            scanline_buffers: [Scanline::new(), Scanline::new()],
            current_scanline_buffer: 0,
            last_scanline: 0,
            graphics_callback: Box::from(graphics_callback),
            controller_callback: Box::from(controller_callback),
            controller_1: Controller::new(),
            controller_2: Controller::new(),
            last_frame: Instant::now(),
            rendering_overhead: RollingAvg::new(60),
            last_60_frames: Instant::now(),
            current_fps: 0,
            frame_counter: 0,
            desired_frame_duration: FRAME_DURATION.mul_f64(speed_multiplier.inv()),
            audio_ring_buffer: Arc::new(Mutex::new(RingBuffer::new())),
            system_palette,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_state(
        state: BusState,
        rom: Rom,
        speed_multiplier: f64,
        graphics_callback: impl GraphicsCallback<'a>,
        controller_callback: impl ControllerCallback<'a>,
        controller_1: Controller,
        controller_2: Controller,
        audio_buffer: AudioBuffer,
        system_palette: SystemPalette,
    ) -> Option<Self> {
        Some(Bus {
            cpu_vram: state.cpu_vram,
            rom: Rom::from_state(rom, state.rom)?,
            ppu: state.ppu,
            apu: state.apu,
            frame: Frame::default(),
            scanline_buffers: [Scanline::new(), Scanline::new()],
            current_scanline_buffer: 0,
            last_scanline: state.last_scanline,
            cycles: state.cycles,
            open_bus: state.open_bus,
            graphics_callback: Box::from(graphics_callback),
            controller_callback: Box::from(controller_callback),
            controller_1,
            controller_2,
            last_frame: Instant::now(),
            rendering_overhead: RollingAvg::new(60),
            last_60_frames: Instant::now(),
            current_fps: 0,
            frame_counter: 0,
            desired_frame_duration: FRAME_DURATION.mul_f64(speed_multiplier.inv()),
            audio_ring_buffer: audio_buffer,
            system_palette,
        })
    }

    pub fn set_speed_multiplayer(&mut self, speed_multiplier: f64) {
        let mut speed_multiplier = speed_multiplier;
        if speed_multiplier <= 0f64 {
            speed_multiplier = f64::INFINITY;
        }
        self.apu
            .as_mut()
            .unwrap()
            .set_speed_multiplayer(speed_multiplier);
        self.desired_frame_duration = FRAME_DURATION.mul_f64(speed_multiplier.inv());
    }

    pub fn in_vblank(&self) -> bool {
        self.ppu.is_in_vertical_blank()
    }

    pub fn get_cycle_count_cpu(&self) -> u64 {
        self.cycles
    }

    pub fn get_cycle_count_ppu(&self) -> (usize, usize) {
        ((self.ppu.scan_line + 1) as usize, self.ppu.cycles)
    }

    pub fn trace_mem_read(&self, addr: u16) -> u8 {
        if let Some(result) = self.ppu.trace_mem_read(addr) {
            return result;
        }
        if let Some(result) = self.rom.mem_read(addr) {
            return result;
        }
        if let Some(result) = self.apu.as_ref().unwrap().trace_mem_read(addr) {
            return result;
        }
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            _ => {
                println!("Reading 0 for unknown address: {addr:X}");
                0
            }
        }
    }

    pub fn trace_mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.trace_mem_read(pos) as u16;
        let hi = self.trace_mem_read(pos + 1) as u16;
        (hi << 8) | lo
    }

    pub fn manual_re_render(&mut self) {
        (self.graphics_callback)(&self.ppu, &self.frame, self.current_fps, &self.rom);
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as u64;
        let mut apu = self.apu.take().unwrap();
        apu.tick(cycles, self);
        self.apu = Some(apu);

        let vblank_before = self.ppu.is_in_vertical_blank();
        let next_scanline = self.ppu.tick(
            &self.system_palette,
            cycles * 3,
            &self.rom,
            &mut self.scanline_buffers,
            self.current_scanline_buffer,
        );
        let vblank_after = self.ppu.is_in_vertical_blank();

        // pre render scanline has index -1
        if next_scanline == PRE_RENDER_SCNALINE {
            return;
        }

        if next_scanline != self.last_scanline && next_scanline < VBLANK_START {
            self.scanline_buffers[self.current_scanline_buffer]
                .write_scanline(&mut self.frame, next_scanline as usize);
            self.last_scanline = next_scanline;
            self.current_scanline_buffer ^= 1;
        }

        if !vblank_before && vblank_after {
            let avg_overhead = Duration::from_nanos(self.rendering_overhead.avg().unwrap_or(0));

            let sleep_duration = self
                .desired_frame_duration
                .checked_sub(self.last_frame.elapsed())
                .and_then(|d| d.checked_sub(avg_overhead))
                .unwrap_or(Duration::ZERO);

            if sleep_duration > Duration::ZERO {
                spin_sleep::sleep(sleep_duration);
            }

            if self.frame_counter.is_multiple_of(60) {
                self.current_fps = (60f64 / self.last_60_frames.elapsed().as_secs_f64()) as u32;
                self.last_60_frames = Instant::now();
            }

            let rendering_start = Instant::now();
            (self.graphics_callback)(&self.ppu, &self.frame, self.current_fps, &self.rom);
            (self.controller_callback)(&mut self.controller_1, &mut self.controller_2, self.cycles);
            let overhead = rendering_start.elapsed().as_nanos() as u64;
            self.rendering_overhead.push(overhead);
            self.last_frame = Instant::now();
            self.frame_counter += 1;
        }
    }
}

pub trait PollNMI {
    fn poll_nmi_status(&mut self) -> bool;
}

pub trait PollIRQ {
    fn poll_irq(&mut self) -> bool;
}

pub trait Mem {
    fn mem_read(&mut self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | lo
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
impl PollNMI for Bus<'_> {
    fn poll_nmi_status(&mut self) -> bool {
        self.ppu.poll_nmi_status()
    }
}

impl PollIRQ for Bus<'_> {
    fn poll_irq(&mut self) -> bool {
        self.apu.as_mut().unwrap().poll_irq()
    }
}

impl Mem for Bus<'_> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        let value = if let Some(result) = self.ppu.mem_read(addr, &self.rom) {
            result
        } else if let Some(result) = self.rom.mem_read(addr) {
            result
        } else if let Some(result) = self.apu.as_mut().unwrap().mem_read(addr, self.open_bus) {
            result
        } else {
            match addr {
                RAM..=RAM_MIRRORS_END => {
                    let mirror_down_addr = addr & 0b00000111_11111111;
                    self.cpu_vram[mirror_down_addr as usize]
                }
                0x4016 => {
                    (self.controller_callback)(
                        &mut self.controller_1,
                        &mut self.controller_2,
                        self.cycles,
                    );
                    (self.open_bus & 0b1110_0000) | (self.controller_1.read() & 0b0001_1111)
                }
                0x4017 => {
                    (self.controller_callback)(
                        &mut self.controller_1,
                        &mut self.controller_2,
                        self.cycles,
                    );
                    (self.open_bus & 0b1110_0000) | (self.controller_2.read() & 0b0001_1111)
                }
                _ => self.open_bus,
            }
        };
        // internal CPU register
        if addr != 0x4015 {
            self.open_bus = value;
        }
        value
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.open_bus = data;
        self.ppu.mem_write(addr, data, &mut self.rom);
        self.rom.mem_write(addr, data);
        self.apu.as_mut().unwrap().mem_write(addr, data);
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            0x4014 => {
                // https://wiki.nesdev.com/w/index.php/PPU_programmer_reference#OAM_DMA_.28.244014.29_.3E_write
                // https://www.nesdev.org/wiki/PPU_OAM#DMA
                // write to oam via dma is directly implemented here instead of using the method from PPU to avoid a buffer and to make it simpler
                if self.cycles.is_multiple_of(2) {
                    self.tick(1);
                } else {
                    self.tick(2);
                }
                let start_address = (data as u16) << 8;
                let mut oam_start = self.ppu.oam_addr;
                for i in 0..256 {
                    self.ppu.oam_data[oam_start as usize] = self.mem_read(start_address + i as u16);
                    oam_start = oam_start.wrapping_add(1);
                    self.tick(2);
                }
            }
            0x4016 => {
                self.controller_1.write(data);
                self.controller_2.write(data);
            }
            _ => (),
        }
    }
}
