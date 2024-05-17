
pub struct ScrollRegister {
    scroll_x: u8,
    scroll_y: u8
}

impl ScrollRegister {
    pub fn new() -> Self {
        Self {
            scroll_x: 0,
            scroll_y: 0,
        }
    }

    pub fn update(&mut self, write_toggle: &mut bool, data: u8) {
        if !*write_toggle {
            self.scroll_x = data;
        } else {
            self.scroll_y = data;
        }
        *write_toggle = !*write_toggle;
    }

    pub fn get_scroll_x(&self) -> u8 {
        self.scroll_x
    }
    
    pub fn get_scroll_y(&self) -> u8 {
        self.scroll_y
    }
}

impl Default for ScrollRegister {
    fn default() -> Self {
        Self::new()
    }
}