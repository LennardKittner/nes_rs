use crate::ppu::t_register::TRegister;

pub struct AddressRegister {
    pub data: u16,
    /// Used for pattern table accesses.
    /// Does not use t register.
    pub data_alt: u16,
}

impl AddressRegister {
    pub fn new() -> Self {
        AddressRegister {
            data: 0,
            data_alt: 0,
        }
    }

    pub fn update_from(&mut self, t: &TRegister) {
        self.data = t.data;

        // mirror down address above 0x3FFF
        if self.data > 0x3FFF {
            self.data &= 0b11_1111_1111_1111;
        }
    }

    pub fn increment(&mut self, inc: u8) {
        self.data = self.data.wrapping_add(inc as u16);

        // mirror down address above 0x3FFF
        if self.data > 0x3FFF {
            self.data &= 0b11_1111_1111_1111;
        }
    }

    pub fn increment_alt(&mut self, inc: u8) {
        self.data_alt = self.data_alt.wrapping_add(inc as u16);

        // mirror down address above 0x3FFF
        if self.data_alt > 0x3FFF {
            self.data_alt &= 0b11_1111_1111_1111;
        }
    }

    pub fn get_inner_tile_y_offset(&self) -> usize {
        ((self.data & 0b111_0000_0000_0000) >> 12) as usize
    }

    pub fn set_inner_tile_y_offset(&mut self, data: u8) {
        let l = ((data % 8) as u16) << 12;
        self.data = (self.data & !0b111_0000_0000_0000) | l;
    }

    pub fn get_tile_x(&self) -> usize {
        (self.data & 0b000_0000_0001_1111) as usize
    }

    pub fn get_tile_y(&self) -> usize {
        ((self.data & 0b000_0011_1110_0000) >> 5) as usize
    }

    pub fn set_tile_y(&mut self, data: u8) {
        let l = ((data % 30) as u16) << 5;
        self.data = (self.data & !0b000_0011_1110_0000) | l;
    }

    pub fn get_name_table(&self) -> usize {
        ((self.data & 0b1100_0000_0000) >> 10) as usize
    }

    pub fn vertical_name_table_overflow(&mut self) {
        self.data ^= 0b10_00000_00000;
    }

    pub fn horizontal_name_table_overflow(&mut self) {
        self.data ^= 0b01_00000_00000;
    }

    pub fn load_x_from(&mut self, t: &TRegister) {
        let tmp = (self.data & !0b100_0001_1111) | (t.data & 0b100_0001_1111);
        self.data = tmp;
    }

    pub fn load_y_from(&mut self, t: &TRegister) {
        let tmp = (self.data & !0b0111_0011_1110_0000) | (t.data & 0b111_1011_1110_0000);
        self.data = tmp;
    }
}

impl Default for AddressRegister {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_setter_test() {
        let mut a = AddressRegister::new();
        a.data = 0b111010011100000;
        let mut b = AddressRegister::new();
        b.set_tile_y(7);
        b.set_inner_tile_y_offset(7);
        b.horizontal_name_table_overflow();

        assert_eq!(a.data, b.data);
    }

    #[test]
    fn test_tile_y_test() {
        let mut a = AddressRegister::new();
        a.data = 0b111010011100000;
        println!(
            "1 x: {} y: {} yi: {} nt: {}",
            a.get_tile_x(),
            a.get_tile_y(),
            a.get_inner_tile_y_offset(),
            a.get_name_table()
        );
        a.set_tile_y((7 + 1) as u8);
        println!(
            "2 x: {} y: {} yi: {} nt: {}",
            a.get_tile_x(),
            a.get_tile_y(),
            a.get_inner_tile_y_offset(),
            a.get_name_table()
        );
        a.horizontal_name_table_overflow();
        println!(
            "3 x: {} y: {} yi: {} nt: {}",
            a.get_tile_x(),
            a.get_tile_y(),
            a.get_inner_tile_y_offset(),
            a.get_name_table()
        );
        a.set_inner_tile_y_offset((7 + 1) as u8);
        println!(
            "4 x: {} y: {} yi: {} nt: {}",
            a.get_tile_x(),
            a.get_tile_y(),
            a.get_inner_tile_y_offset(),
            a.get_name_table()
        );
    }

    #[test]
    fn test_load_x_from() {
        let mut a = AddressRegister::new();
        a.data = 0xFFFF;
        let mut t = TRegister::new();
        t.data = 0b10100010_01011001;
        a.load_x_from(&t);
        assert_eq!(a.data, 0b11111011_11111001)
    }

    #[test]
    fn test_load_y_from() {
        let mut a = AddressRegister::new();
        a.data = 0xFFFF;
        let mut t = TRegister::new();
        t.data = 0b10100010_01011001;
        a.load_y_from(&t);
        assert_eq!(a.data, 0b10101110_01011111)
    }
}
