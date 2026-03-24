use serde::{Deserialize, Serialize};

use crate::{
    bus::{AudioBuffer, Bus, BusState, ControllerCallback, GraphicsCallback},
    controller::Controller,
    cpu::{CPUState, CPU},
    ppu::{palette::SystemPalette, PPU},
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

//TODO: use this as the interface to the outside
//TODO: implement RomState

#[derive(Debug)]
pub struct NES<'a> {
    pub cpu: CPU<'a>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NESState {
    cpu: CPUState,
    bus: BusState,
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
    pub fn new<GF, C1F>(
        rom: Rom,
        system_palette: SystemPalette,
        speed_multiplier: f64,
        graphics_callback: GF,
        controller_callback: C1F,
    ) -> (NES<'a>, AudioBuffer)
    where
        GF: FnMut(&PPU, &Frame, u32, &Rom) + 'a,
        C1F: FnMut(&mut Controller, &mut Controller) + 'a,
    {
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
        (NES { cpu }, audio_buffer)
    }

    pub fn from_state(
        state: NESState,
        rom: Rom,
        speed_multiplier: f64,
        graphics_callback: impl GraphicsCallback<'a>,
        controller_callback: impl ControllerCallback<'a>,
        audio_buffer: AudioBuffer,
    ) -> Option<Self> {
        let bus_tmp = Bus::from_state(
            state.bus,
            rom,
            speed_multiplier,
            graphics_callback,
            controller_callback,
            audio_buffer,
        )?;
        Some(NES {
            cpu: CPU::from_state(state.cpu, bus_tmp),
        })
    }

    pub fn set_speed_multiplayer(&mut self, speed_multiplier: f64) {
        self.cpu.bus.set_speed_multiplayer(speed_multiplier);
    }

    pub fn step(&mut self) -> bool {
        self.cpu.step()
    }

    pub fn manual_re_render(&mut self) {
        self.cpu.bus.manual_re_render();
    }
}
