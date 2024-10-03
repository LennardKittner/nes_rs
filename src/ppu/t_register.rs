pub struct TRegister {
    pub data: u16,
}

impl TRegister {
    pub fn new() -> Self {
        TRegister { data: 0 }
    }

    pub fn update_addr_write(&mut self, data: u8, write_toggle: bool) {
        if !write_toggle {
            self.data = (self.data & !0b00111111_00000000) | (((data & 0b00111111) as u16) << 8);
            self.data &= 0b10111111_11111111;
        } else {
            self.data = (self.data & !0b11111111) | data as u16;
        }
    }

    pub fn update_ctrl_write(&mut self, data: u8) {
        self.data = (self.data & !(0b11 << 10)) | (((data & 0b11) as u16) << 10);
    }

    pub fn update_scroll_write(&mut self, data: u8, write_toggle: bool) {
        if !write_toggle {
            self.data = (self.data & !0b11111) | ((data & 0b11111000) as u16 >> 3);
        } else {
            let d1 = (data as u16 & 0b00000111) << 12;
            let d2 = (data as u16 & 0b11000000) << 2;
            let d3 = (data as u16 & 0b00111000) << 2;
            let d = d1 | d2 | d3;
            self.data = (self.data & !0b11110011_11100000) | d;
        }
    }
}

impl Default for TRegister {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_update_ctrl_write() {
        let mut t = TRegister::new();
        t.update_ctrl_write(10);
        let mut tt = TRegister::new();
        tt.update_ctrl_write(01);
        let mut ttt = TRegister::new();
        ttt.update_ctrl_write(11);
        let mut tttt = TRegister::new();
        tttt.update_ctrl_write(11);
        tttt.update_ctrl_write(00);
        assert_eq!(t.data, 0b1000_00000000);
        assert_eq!(tt.data, 0b100_00000000);
        assert_eq!(ttt.data, 0b1100_00000000);
        assert_eq!(tttt.data, 0);
    }

    #[test]
    fn test_update_scroll_write() {
        let mut t = TRegister::new();
        t.update_scroll_write(0b010110101, false);
        assert_eq!(t.data, 0b010110);

        let mut tt = TRegister::new();
        tt.update_scroll_write(0b10110101, true);
        println!("{:8b}", tt.data & 0b00000000_11100000);
        assert_eq!(tt.data & 0b00000000_11100000, 0b00000000_11000000);
        assert_eq!(tt.data & 0b00000011_00000000, 0b00000010_00000000);
        assert_eq!(tt.data & 0b01110000_00000000, 0b01010000_00000000);
    }

    #[test]
    fn test_update_addr_write() {
        let mut t = TRegister::new();
        t.data = 0b01000000_00000000;
        t.update_addr_write(0b10110101, false);
        assert_eq!(t.data, 0b110101_00000000);

        let mut tt = TRegister::new();
        tt.data = 0xFFFF;
        tt.update_addr_write(0b10110101, true);
        assert_eq!(tt.data, 0b11111111_10110101);
    }
}
