
pub struct ScrollRegister {
    scroll_x: u8,
    scroll_y: u8,
    w_latch: bool
}

impl ScrollRegister {
    pub fn new() -> Self {
        Self {
            scroll_x: 0,
            scroll_y: 0,
            w_latch: false,
        }
    }

    pub fn update(&mut self, data: u8) {
        if !self.w_latch {
            self.scroll_x = data;
        } else {
            self.scroll_y = data;
        }
        self.w_latch = !self.w_latch;
    }

    pub fn reset_letch(&mut self) {
        self.w_latch = false;
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