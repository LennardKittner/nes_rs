use crate::apu::APU;
use crate::controller::Controller;
use crate::ppu::palette::SystemPalette;
use crate::ppu::PPU;
use crate::rendering::fps_frame::FPSFrame;
use crate::rendering::{frame::Frame, render, scanline::Scanline};
use crate::ring_buffer::RingBuffer;
use crate::rolling_avg::RollingAvg;
use crate::rom::Rom;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const FRAME_DURATION: Duration = Duration::from_nanos(16666667);

pub const AUDIO_BUFFER_SIZE: usize = 44100;
type GraphicsCallback<'a> = Box<dyn FnMut(&PPU, &Frame, &FPSFrame) + 'a>;
type ControllerCallback<'a> = Box<dyn FnMut(&mut Controller, &mut Controller) + 'a>;

pub struct Bus<'a> {
    cpu_vram: [u8; 2048],
    rom: Rom,
    ppu: PPU,
    apu: Option<APU>,
    frame: Frame,
    fps_frame: FPSFrame,
    current_scanline: Scanline,
    last_scanline: u16,
    cycles: usize,
    graphics_callback: GraphicsCallback<'a>,
    controller_callback: ControllerCallback<'a>,
    controller_1: Controller,
    controller_2: Controller,
    last_frame: Instant,
    rendering_overhead: RollingAvg<u64>,
    fps: RollingAvg<f64>,
    frame_counter: u64,
    pub audio_ring_buffer: Arc<Mutex<RingBuffer<f32, AUDIO_BUFFER_SIZE>>>, // 1s of audio
}

impl<'a> Bus<'a> {
    pub fn new<GF, C1F>(
        rom: Rom,
        system_palette: SystemPalette,
        graphics_callback: GF,
        controller_callback: C1F,
    ) -> Bus<'a>
    where
        GF: FnMut(&PPU, &Frame, &FPSFrame) + 'a,
        C1F: FnMut(&mut Controller, &mut Controller) + 'a,
    {
        let ppu = PPU::new(rom.screen_mirroring, system_palette);
        Bus {
            cpu_vram: [0; 2048],
            rom,
            cycles: 0,
            ppu,
            apu: Some(APU::new()),
            frame: Frame::default(),
            fps_frame: FPSFrame::new(0, 0xA, [0x0F, 0x30, 0x21, 0x0F]),
            current_scanline: Scanline::new(),
            last_scanline: 0,
            graphics_callback: Box::from(graphics_callback),
            controller_callback: Box::from(controller_callback),
            controller_1: Controller::new(),
            controller_2: Controller::new(),
            last_frame: Instant::now(),
            rendering_overhead: RollingAvg::new(60),
            fps: RollingAvg::new(60),
            frame_counter: 0,
            audio_ring_buffer: Arc::new(Mutex::new(RingBuffer::new())),
        }
    }
    
    pub fn get_cycle_count_cpu(&self) -> usize {
        self.cycles
    }
    
    pub fn get_cycle_count_ppu(&self) -> (usize, usize) {
        (self.ppu.scan_line as usize, self.ppu.cycles)
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        let mut addr = addr - 0x8000;
        if self.rom.prg_rom_len() == 0x4000 && addr >= 0x4000 {
            addr %= 0x4000;
        }
        self.rom.read_prg_rom(addr)
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;
        let mut apu = self.apu.take().unwrap();
        apu.tick(cycles, self);
        self.apu = Some(apu);

        let vblank_before = self.ppu.is_in_vertical_blank();
        let next_scanline = self.ppu.tick(cycles * 3);
        let vblank_after = self.ppu.is_in_vertical_blank();

        if next_scanline != self.last_scanline && next_scanline <= 240 {
            render(
                &mut self.ppu,
                &self.rom,
                &mut self.current_scanline,
                next_scanline as usize,
            );
            self.current_scanline
                .write_scanline(&mut self.frame, next_scanline as usize);
            self.current_scanline.clear();
            self.last_scanline = next_scanline;
        }

        if !vblank_before && vblank_after {
            let avg_overhead = Duration::from_nanos(self.rendering_overhead.avg().unwrap_or(0));
            let sleep_duration = FRAME_DURATION
                .checked_sub(self.last_frame.elapsed())
                .and_then(|d| d.checked_sub(avg_overhead))
                .unwrap_or(Duration::ZERO);

            if sleep_duration > Duration::ZERO {
                spin_sleep::sleep(sleep_duration);
            }

            let fps = 1.0 / self.last_frame.elapsed().as_secs_f64();
            self.fps.push(fps);
            if self.frame_counter % 60 == 0 {
                let avg = self.fps.avg().unwrap_or_default() as usize;
                self.fps_frame
                    .update(&self.rom, 1, avg, self.ppu.get_universal_background_color());
            }

            let rendering_start = Instant::now();
            (self.graphics_callback)(&self.ppu, &self.frame, &self.fps_frame);
            (self.controller_callback)(&mut self.controller_1, &mut self.controller_2);
            let overhead = rendering_start.elapsed().as_nanos() as u64;
            if self.frame_counter > 300 {
                // skip initial high overhead
                self.rendering_overhead.push(overhead);
            }
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
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const CARTRIDGE_START: u16 = 0x8000;
const CARTRIDGE_END: u16 = 0xFFFF;
const APU_REGISTERS_START: u16 = 0x4000;
const APU_REGISTERS_END: u16 = 0x4013;

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
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                //TODO: maybe detect tracing
                //panic!("Attempt to read from write-only PPU address {:x}", addr);
                0
            }
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read_data(self.rom.get_current_chr_rom()),
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_read(mirror_down_addr)
            }
            APU_REGISTERS_START..=APU_REGISTERS_END => 0,
            0x4015 => self.apu.as_mut().unwrap().get_status(),
            0x4016 => {
                (self.controller_callback)(&mut self.controller_1, &mut self.controller_2);
                self.controller_1.read()
            }
            0x4017 => {
                (self.controller_callback)(&mut self.controller_1, &mut self.controller_2);
                self.controller_2.read()
            }
            CARTRIDGE_START..=CARTRIDGE_END => self.read_prg_rom(addr),
            _ => {
                println!("Ignoring mem read at {addr:x}");
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            0x2000 => self.ppu.write_to_ctrl(data),
            0x2001 => self.ppu.write_to_mask(data),
            0x2002 => panic!("write to PPU status register"),
            0x2003 => self.ppu.write_to_addr(data),
            0x2004 => self.ppu.write_to_data(data, self.rom.get_current_chr_ram()),
            0x2005 => self.ppu.write_to_scroll(data),
            0x2006 => self.ppu.write_to_addr(data),
            0x2007 => self.ppu.write_to_data(data, self.rom.get_current_chr_ram()),
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_write(mirror_down_addr, data);
            }
            //APU:

            // pulse 1
            0x4000 => self.apu.as_mut().unwrap().set_pulse1_DLCV(data),
            0x4001 => self.apu.as_mut().unwrap().set_pulse1_EPNS(data),
            0x4002 => self.apu.as_mut().unwrap().set_pulse1_timer_low(data),
            0x4003 => self.apu.as_mut().unwrap().set_pulse1_LT(data),

            // pulse 2
            0x4004 => self.apu.as_mut().unwrap().set_pulse2_DLCV(data),
            0x4005 => self.apu.as_mut().unwrap().set_pulse2_EPNS(data),
            0x4006 => self.apu.as_mut().unwrap().set_pulse2_timer_low(data),
            0x4007 => self.apu.as_mut().unwrap().set_pulse2_LT(data),

            // triangle
            0x4008 => self.apu.as_mut().unwrap().set_triangle_CR(data),
            0x4009 => (), // unused
            0x400A => self.apu.as_mut().unwrap().set_triangle_timer_low(data),
            0x400B => self.apu.as_mut().unwrap().set_triangle_LT(data),

            // noise
            0x400C => self.apu.as_mut().unwrap().set_noise_LCV(data),
            0x400D => (), // unused
            0x400E => self.apu.as_mut().unwrap().set_noise_LP(data),
            0x400F => self
                .apu
                .as_mut()
                .unwrap()
                .set_noise_length_counter_load(data),

            // DMC
            0x4010 => self.apu.as_mut().unwrap().set_DMC_ILR(data),
            0x4011 => self.apu.as_mut().unwrap().set_DMC_load_counter(data),
            0x4012 => self.apu.as_mut().unwrap().set_DMC_sample_address(data),
            0x4013 => self.apu.as_mut().unwrap().set_DMC_sample_length(data),

            0x4014 => {
                // https://wiki.nesdev.com/w/index.php/PPU_programmer_reference#OAM_DMA_.28.244014.29_.3E_write
                // https://www.nesdev.org/wiki/PPU_OAM#DMA
                // write to oam via dma is directly implemented here instead of using the method from PPU to avoid a buffer and to make it simpler
                if self.cycles % 2 == 0 {
                    self.tick(1);
                } else {
                    self.tick(2);
                }
                let start_address = (data as u16) << 8;
                for i in 0..256 {
                    self.ppu.oam_data[self.ppu.oam_addr as usize] =
                        self.mem_read(start_address + i as u16);
                    self.ppu.oam_addr = self.ppu.oam_addr.wrapping_add(1);
                    self.tick(2);
                }
            }
            0x4015 => self.apu.as_mut().unwrap().set_status(data),
            0x4016 => {
                self.controller_1.write(data);
                self.controller_2.write(data);
            }
            0x4017 => self.apu.as_mut().unwrap().set_frame_counter(data),
            CARTRIDGE_START..=CARTRIDGE_END => self.rom.mapper_register_write(addr, data),
            _ => {
                println!("Ignoring mem write at 0x{addr:X}");
            }
        }
    }
}

//TODO: tests
