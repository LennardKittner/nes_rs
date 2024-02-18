pub struct AddressRegister {
    value: (u8, u8), // 16bit address high byte first, low bytes second
    hi_ptr: bool,
}

impl AddressRegister {
    pub fn new() -> Self {
        AddressRegister {
            value: (0, 0),
            hi_ptr: true,
        }
    }

    fn set(&mut self, data: u16) {
        self.value.0 = (data >> 8) as u8;
        self.value.1 = (data & 0xFF) as u8;
    }

    pub fn update(&mut self, data: u8) {
        if self.hi_ptr {
            self.value.0 = data;
        } else {
            self.value.1 = data;
        }

        // mirror down address above 0x3FFF
        if self.get() > 0x3FFF {
            self.set(self.get() & 0b11_1111_1111_1111)
        }
        self.hi_ptr = !self.hi_ptr;
    }

    pub fn increment(&mut self, inc: u8) {
        let carry;
        (self.value.1, carry) = self.value.1.overflowing_add(inc);
        if carry {
            self.value.0 = self.value.0.wrapping_add(1);
        }

        // mirror down address above 0x3FFF
        if self.get() > 0x3FFF {
            self.set(self.get() & 0b11_1111_1111_1111)
        }
    }

    pub fn reset_latch(&mut self) {
        self.hi_ptr = true;
    }

    pub fn get(&self) -> u16 {
        (self.value.0 as u16) << 8 | (self.value.1 as u16)
    }
}