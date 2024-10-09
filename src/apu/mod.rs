use crate::apu::data_modulation_channel::DataModulationChannel;
use crate::apu::noise_generator::NoiseGenerator;
use crate::apu::pulse_generator::{PulseGenerator, PulseGeneratorID};
use crate::apu::triangle_generator::TriangleGenerator;
use crate::bus::{Bus, PollIRQ};

mod data_modulation_channel;
mod envelope;
mod length_counter;
mod noise_generator;
mod pulse_generator;
mod sweep_unit;
mod timer;
mod triangle_generator;

#[derive(Eq, PartialEq)]
enum FrameCounterMode {
    MODE5STEP,
    MODE4STEP,
}

pub struct APU {
    pulse_generator1: PulseGenerator,
    pulse_generator2: PulseGenerator,
    noise_generator: NoiseGenerator,
    triangle_generator: TriangleGenerator,
    data_modulation_channel: DataModulationChannel,
    cycle_in_frame: usize,
    enable_interrupt: bool,
    frame_counter_mode: FrameCounterMode,
    outstanding_interrupt: bool,
}

impl PollIRQ for APU {
    fn poll_irq(&mut self) -> bool {
        if self.outstanding_interrupt {
            self.outstanding_interrupt = false;
            true
        } else {
            false
        }
    }
}

impl APU {
    pub fn new() -> APU {
        Self {
            pulse_generator1: PulseGenerator::new(PulseGeneratorID::One),
            pulse_generator2: PulseGenerator::new(PulseGeneratorID::Two),
            noise_generator: NoiseGenerator::new(),
            triangle_generator: TriangleGenerator::new(),
            data_modulation_channel: DataModulationChannel::new(),
            cycle_in_frame: 0,
            enable_interrupt: false,
            frame_counter_mode: FrameCounterMode::MODE4STEP,
            outstanding_interrupt: false,
        }
    }

    pub fn tick(
        &mut self,
        cycles: u8,
        bus: &mut Bus,
        //ring_buffer: Arc<Mutex<RingBuffer<f32, AUDIO_BUFFER_SIZE>>>,
    ) {
        for _ in 0..cycles {
            self.cycle_in_frame += 1;
            self.triangle_generator.tick(); // tick every cpu cycle
            if self.cycle_in_frame % 2 == 1 {
                continue;
            }
            if self.frame_counter_mode == FrameCounterMode::MODE5STEP {
                self.tick_5_step()
            } else {
                self.tick_4_step()
            }

            self.pulse_generator1.tick();
            self.pulse_generator2.tick();
            self.noise_generator.tick();
            self.data_modulation_channel.tick(bus);
            self.outstanding_interrupt = self.data_modulation_channel.poll_irq();
            bus.audio_ring_buffer.lock().unwrap().push(self.get_output());
        }
    }

    fn tick_all_channels_half_frame(&mut self) {
        self.pulse_generator1.tick_half_frame();
        self.pulse_generator2.tick_half_frame();
        self.noise_generator.tick_half_frame();
        self.triangle_generator.tick_half_frame();
    }

    fn tick_all_channels_quater_frame(&mut self) {
        self.pulse_generator1.tick_quarter_frame();
        self.pulse_generator2.tick_quarter_frame();
        self.noise_generator.tick_quarter_frame();
        self.triangle_generator.tick_quarter_frame();
    }

    fn tick_4_step(&mut self) {
        match self.cycle_in_frame {
            c if c >= 14914 => {
                self.tick_all_channels_half_frame();
                if self.enable_interrupt {
                    self.outstanding_interrupt = true;
                }
                self.cycle_in_frame = 0;
            }
            c if c >= 11185 => self.tick_all_channels_quater_frame(),
            c if c >= 7456 => self.tick_all_channels_half_frame(),
            c if c >= 3728 => self.tick_all_channels_quater_frame(),
            _ => {}
        }
    }

    fn tick_5_step(&mut self) {
        match self.cycle_in_frame {
            c if c >= 18640 => {
                self.tick_all_channels_half_frame();
                self.cycle_in_frame = 0;
            }
            c if c >= 14914 => { /* Do Nothing */ }
            c if c >= 11185 => self.tick_all_channels_quater_frame(),
            c if c >= 7456 => self.tick_all_channels_half_frame(),
            c if c >= 3728 => self.tick_all_channels_quater_frame(),
            _ => {}
        }
    }

    pub fn get_output(&self) -> f32 {
        let pulse1 = self.pulse_generator1.get_output();
        let pulse2 = self.pulse_generator2.get_output();
        let pulse_out = if pulse1 + pulse2 <= 0f32 {
            0f32
        } else {
            95.88f32 / ((8128f32 / (pulse1 + pulse2)) + 100f32)
        };

        let noise = self.noise_generator.get_output();
        let triangle = self.triangle_generator.get_output();
        let dmc = self.data_modulation_channel.get_output();
        let tnd_out = if noise <= 0f32 && triangle <= 0f32 && dmc <= 0f32 {
            0f32
        } else {
            159.79f32
                / (1f32 / ((triangle / 8227f32) + (noise / 12241f32) + (dmc / 22638f32)) + 100f32)
        };

        pulse_out + tnd_out
    }

    /// IF-D NT21
    /// DMC interrupt (I), frame interrupt (F), DMC active (D), length counter > 0 (N/T/2/1)
    pub fn get_status(&mut self) -> u8 {
        let mut status = 0;
        if self.pulse_generator1.is_active() {
            status |= 0b0000_0001;
        }
        if self.pulse_generator2.is_active() {
            status |= 0b0000_0010;
        }
        if self.triangle_generator.is_active() {
            status |= 0b0000_0100;
        }
        if self.noise_generator.is_active() {
            status |= 0b0000_1000;
        }
        if self.data_modulation_channel.get_bytes_remaining() != 0 {
            status |= 0b0001_0000;
        }
        if self.enable_interrupt {
            status |= 0b0100_0000;
        }
        if self.data_modulation_channel.is_interrupt_enabled() {
            status |= 0b1000_0000;
        }
        //TODO: If an interrupt flag was set at the same moment of the read, it will read back as 1 but it will not be cleared. what?
        self.enable_interrupt = false;
        status
    }

    /// ---D NT21
    /// Enable DMC (D), noise (N), triangle (T), and pulse channels (2/1)
    pub fn set_status(&mut self, value: u8) {
        self.pulse_generator1
            .set_length_counter_enabled(value & 0b0000_0001 != 0);
        self.pulse_generator2
            .set_length_counter_enabled(value & 0b0000_0010 != 0);
        self.triangle_generator
            .set_length_counter_enabled(value & 0b0000_0100 != 0);
        self.noise_generator
            .set_length_counter_enabled(value & 0b0000_1000 != 0);
        if value & 0b0001_0000 == 0 {
            self.data_modulation_channel.set_bytes_remaining(0);
        } else {
            if self.data_modulation_channel.get_bytes_remaining() == 0 {
                self.data_modulation_channel.restart();
            }
            self.data_modulation_channel.set_irq_enable(false);
        }
    }

    /// MI-- ----
    /// Mode (M, 0 = 4-step, 1 = 5-step), IRQ inhibit flag (I)
    pub fn set_frame_counter(&mut self, value: u8) {
        if value & 0b1000_0000 != 0 {
            self.frame_counter_mode = FrameCounterMode::MODE4STEP;
        } else {
            self.frame_counter_mode = FrameCounterMode::MODE5STEP;
        }
        self.enable_interrupt = value & 0b0100_0000 == 0;
    }

    /// DDLC VVVV
    /// Duty (D), envelope loop / length counter halt (L), constant volume (C), volume/envelope (V)
    #[allow(non_snake_case)]
    pub fn set_pulse1_DLCV(&mut self, value: u8) {
        self.pulse_generator1.set_duty((value & 0b1100_0000) >> 6);
        let l = value & 0b0010_0000 != 0;
        self.pulse_generator1.set_length_counter_halt(l);
        self.pulse_generator1
            .set_envelope_parameters(l, value & 0b0001_0000 != 0, value & 0x0F);
    }

    /// DDLC VVVV
    /// Duty (D), envelope loop / length counter halt (L), constant volume (C), volume/envelope (V)
    #[allow(non_snake_case)]
    pub fn set_pulse2_DLCV(&mut self, value: u8) {
        self.pulse_generator2.set_duty((value & 0b1100_0000) >> 6);
        let l = value & 0b0010_0000 != 0;
        self.pulse_generator2.set_length_counter_halt(l);
        self.pulse_generator2
            .set_envelope_parameters(l, value & 0b0001_0000 != 0, value & 0x0F);
    }

    /// EPPP NSSS
    /// Sweep unit: enabled (E), period (P), negate (N), shift (S)
    #[allow(non_snake_case)]
    pub fn set_pulse1_EPNS(&mut self, value: u8) {
        self.pulse_generator1.set_sweep_parameters(
            value & 0b1000_0000 != 0,
            value & 0b0000_1000 != 0,
            value & 0b0000_0111,
            (value & 0b0111_0000) >> 4,
        )
    }

    /// EPPP NSSS
    /// Sweep unit: enabled (E), period (P), negate (N), shift (S)
    #[allow(non_snake_case)]
    pub fn set_pulse2_EPNS(&mut self, value: u8) {
        self.pulse_generator2.set_sweep_parameters(
            value & 0b1000_0000 != 0,
            value & 0b0000_1000 != 0,
            value & 0b0000_0111,
            (value & 0b0111_0000) >> 4,
        )
    }

    /// TTTT TTTT
    /// Timer low (T)
    pub fn set_pulse1_timer_low(&mut self, value: u8) {
        self.pulse_generator1.set_timer_lower(value)
    }

    /// TTTT TTTT
    /// Timer low (T)
    pub fn set_pulse2_timer_low(&mut self, value: u8) {
        self.pulse_generator2.set_timer_lower(value)
    }

    /// LLLL LTTT
    /// Length counter load (L), timer high (T)
    #[allow(non_snake_case)]
    pub fn set_pulse1_LT(&mut self, value: u8) {
        self.pulse_generator1.set_length_counter_value(value >> 3);
        self.pulse_generator1.set_timer_upper(value & 0x7)
    }

    /// LLLL LTTT
    /// Length counter load (L), timer high (T)
    #[allow(non_snake_case)]
    pub fn set_pulse2_LT(&mut self, value: u8) {
        self.pulse_generator2.set_length_counter_value(value >> 3);
        self.pulse_generator2.set_timer_upper(value & 0x7)
    }

    /// CRRR RRRR
    /// Length counter halt / linear counter control (C), linear counter load (R)
    #[allow(non_snake_case)]
    pub fn set_triangle_CR(&mut self, value: u8) {
        self.triangle_generator
            .set_length_counter_halt(value & 0b1000_0000 != 0);
        self.triangle_generator
            .set_control_flag(value & 0b1000_0000 != 0);
        self.triangle_generator
            .set_counter_reload_value(value & 0b0111_1111);
    }

    /// TTTT TTTT
    /// Timer low (T)
    pub fn set_triangle_timer_low(&mut self, value: u8) {
        self.triangle_generator.set_timer_lower(value);
    }

    /// LLLL LTTT
    /// Length counter load (L), timer high (T), set linear counter reload flag
    #[allow(non_snake_case)]
    pub fn set_triangle_LT(&mut self, value: u8) {
        self.triangle_generator.set_timer_upper(value & 0b0000_0111);
        self.triangle_generator.set_length_counter_value(value >> 3);
    }

    /// --LC VVVV
    /// Envelope loop / length counter halt (L), constant volume (C), volume/envelope (V)
    #[allow(non_snake_case)]
    pub fn set_noise_LCV(&mut self, value: u8) {
        let l = value & 0b0010_0000 != 0;
        self.noise_generator.set_length_counter_halt(l);
        self.noise_generator
            .set_envelope_parameters(l, value & 0b0001_0000 != 0, value & 0x0F);
    }

    /// L--- PPPP
    /// Loop noise (L), noise period (P)
    #[allow(non_snake_case)]
    pub fn set_noise_LP(&mut self, value: u8) {
        self.noise_generator.set_loop_mode(value & 0b1000_0000 != 0);
        self.noise_generator.set_period(value & 0xF);
    }

    /// LLLL L---
    /// Length counter load (L)
    pub fn set_noise_length_counter_load(&mut self, value: u8) {
        self.noise_generator.set_length_counter_value(value >> 3);
    }

    /// IL-- RRRR
    /// IRQ enable (I), loop (L), frequency (R)
    #[allow(non_snake_case)]
    pub fn set_DMC_ILR(&mut self, value: u8) {
        self.data_modulation_channel
            .set_irq_enable(value & 0b1000_0000 != 0);
        self.data_modulation_channel
            .set_loop_flag(value & 0b0100_0000 != 0);
        self.data_modulation_channel
            .set_output_rate(value & 0b0000_1111);
    }

    /// -DDD DDDD
    /// Load counter (D)
    #[allow(non_snake_case)]
    pub fn set_DMC_load_counter(&mut self, value: u8) {
        self.data_modulation_channel
            .direct_load(value & 0b0111_1111)
    }

    /// AAAA AAAA
    /// Sample address (A)
    #[allow(non_snake_case)]
    pub fn set_DMC_sample_address(&mut self, value: u8) {
        self.data_modulation_channel.set_sample_address(value);
    }

    /// LLLL LLLL
    /// Sample length (L)
    #[allow(non_snake_case)]
    pub fn set_DMC_sample_length(&mut self, value: u8) {
        self.data_modulation_channel.set_sample_length(value);
    }
}
