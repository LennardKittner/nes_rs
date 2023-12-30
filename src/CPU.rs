#[allow(non_snake_case)]
pub struct CPU {
    register_a: u8,
    register_x: u8,
    status: u8,
    program_counter: u16,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            status: 0,
            program_counter: 0,
        }
    }

    pub fn interpret(&mut self, program: &[u8]) {
        self.program_counter = 0;
        loop {
            let op_code = program[self.program_counter as usize];
            self.program_counter += 1;

            match op_code {
                0xA9 => {
                    let value = program[self.program_counter as usize];
                    self.program_counter += 1;
                    self.lda(value);
                },
                0xAA => self.tax(),
                0xE8 => self.inx(),
                0x00 => return,
                _ => todo!("NOT IMPLEMENTED")
            }
        }
    }

    fn update_zero_and_negative_flags(&mut self, value: u8) {
        if value == 0 {
            self.status |= 0b0000_0010;
        } else {
            self.status &= 0b1111_1101;
        }

        if value & 0b1000_0000 != 0 {
            self.status |= 0b1000_0000;
        } else {
            self.status &= 0b0111_1111;
        }
    }

    fn lda(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.checked_add(1).unwrap_or(0);
        self.update_zero_and_negative_flags(self.register_x);
    }
}

#[test]
fn test_5_ops_working_together() {
    let mut cpu = CPU::new();
    cpu.interpret(&vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
    assert_eq!(cpu.register_x, 0xc1);
}

#[test]
fn test_inx_overflow() {
    let mut cpu = CPU::new();
    cpu.register_x = 0xff;
    cpu.interpret(&vec![0xe8, 0xe8, 0x00]);
    assert_eq!(cpu.register_x, 1);
}

#[test]
fn test_tax() {
    let mut cpu = CPU::new();
    cpu.register_a = 10;
    cpu.interpret(&vec![0xaa, 0x00]);
    assert_eq!(cpu.register_x, 10);
}