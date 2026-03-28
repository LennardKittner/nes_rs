// TODO:
// #![warn(missing_docs)]
use serde::{Deserialize, Serialize};

use crate::{
    bus::{AudioBuffer, Bus, BusState, ControllerCallback, GraphicsCallback},
    controller::Controller,
    cpu::{CPUState, CPU},
    ppu::palette::SystemPalette,
    rendering::frame::Frame,
    rom::Rom,
};

mod apu;
pub mod bus;
pub mod controller;
pub mod cpu;
mod mappers;
pub mod ppu;
pub mod rendering;
pub mod ring_buffer;
mod rolling_avg;
pub mod rom;

#[derive(Debug)]
pub struct NES<'a> {
    speed_multiplier: f64,
    cpu: CPU<'a>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NESState {
    cpu: CPUState,
    bus: BusState,
}

impl NESState {
    pub fn get_rom_hash(&self) -> u64 {
        self.bus.get_rom_hash()
    }
}

impl<'a> Serialize for NES<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        NESState {
            cpu: CPUState::new(&self.cpu),
            bus: BusState::new(&self.cpu.bus),
        }
        .serialize(serializer)
    }
}

impl<'a> NES<'a> {
    pub fn new(
        rom: Rom,
        system_palette: SystemPalette,
        speed_multiplier: f64,
        graphics_callback: impl GraphicsCallback<'a>,
        controller_callback: impl ControllerCallback<'a>,
    ) -> (NES<'a>, AudioBuffer) {
        let bus = Bus::new(
            rom,
            system_palette,
            speed_multiplier,
            graphics_callback,
            controller_callback,
        );
        let audio_buffer = bus.audio_ring_buffer.clone();
        let mut cpu = CPU::new_with_bus(bus);
        cpu.reset();
        (
            NES {
                speed_multiplier,
                cpu,
            },
            audio_buffer,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_state(
        state: NESState,
        rom: Rom,
        speed_multiplier: f64,
        graphics_callback: impl GraphicsCallback<'a>,
        controller_callback: impl ControllerCallback<'a>,
        controller_1: Controller,
        contorller_2: Controller,
        audio_buffer: AudioBuffer,
        system_palette: SystemPalette,
    ) -> Option<Self> {
        let bus_tmp = Bus::from_state(
            state.bus,
            rom,
            speed_multiplier,
            graphics_callback,
            controller_callback,
            controller_1,
            contorller_2,
            audio_buffer,
            system_palette,
        )?;
        Some(NES {
            speed_multiplier,
            cpu: CPU::from_state(state.cpu, bus_tmp),
        })
    }

    pub fn replace_state(self, state: NESState) -> Option<Self> {
        let speed_multiplier = self.speed_multiplier;
        let rom = self.cpu.bus.rom;
        let graphics_callback = self.cpu.bus.graphics_callback;
        let controller_callback = self.cpu.bus.controller_callback;
        let audio_buffer = self.cpu.bus.audio_ring_buffer;
        let system_palette = self.cpu.bus.system_palette;
        let controller_1 = self.cpu.bus.controller_1;
        let controller_2 = self.cpu.bus.controller_2;

        let rom_hash = state.get_rom_hash();
        if rom_hash != rom.rom_hash {
            eprintln!("The hash of the current rom and the rom which was played during save state creation missmatch!\nHave fun :)")
        }

        let bus_tmp = Bus::from_state(
            state.bus,
            rom,
            speed_multiplier,
            graphics_callback,
            controller_callback,
            controller_1,
            controller_2,
            audio_buffer,
            system_palette,
        )?;
        Some(NES {
            speed_multiplier,
            cpu: CPU::from_state(state.cpu, bus_tmp),
        })
    }

    pub fn set_speed_multiplayer(&mut self, speed_multiplier: f64) {
        self.speed_multiplier = speed_multiplier;
        self.cpu.bus.set_speed_multiplayer(speed_multiplier);
    }

    pub fn step(&mut self) -> bool {
        self.cpu.step()
    }

    pub fn manual_re_render(&mut self) {
        self.cpu.bus.manual_re_render();
    }

    pub fn in_vblank(&self) -> bool {
        self.cpu.bus.in_vblank()
    }

    pub fn get_frame_counter(&self) -> u64 {
        self.cpu.bus.frame_counter
    }

    pub fn get_current_frame(&self) -> Frame {
        self.cpu.bus.frame.clone()
    }

    /// get the current CPU cycle
    pub fn get_cylce(&self) -> u64 {
        self.cpu.bus.get_cycle_count_cpu() as u64
    }
}
