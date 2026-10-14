//! Simulates physical video card

use sdl3::{gpu::Texture, pixels::PixelFormat, surface::Surface};

pub struct VideoState {
    state: Vec<Option<u8>>,
}

impl VideoState {
    pub fn new(len: usize) -> Self {
        Self {
            state: vec![None; len],
        }
    }

    pub fn memory(&self) -> &Vec<Option<u8>> {
        &self.state
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        println!("writing {val:x} at {addr:x} to ram");
        self.state[addr as usize] = Some(val);
    }

    pub fn read(&self, addr: u32) -> Option<u8> {
        self.state[addr as usize]
    }

    pub fn create_surface_data(&self) -> Vec<u8> {
        let mut data = Vec::new();

        let char_width = 40;
        let width = char_width;
        let height = 240;

        for x in 0..40 {
            for y in 0..height {
                let addr = (x + 1) | (y << 6);
                let color_value = self.memory()[addr].unwrap_or(0);
                let char_value = self.memory()[addr | (1 << 14)].unwrap_or(0);
                let bg_color = color_value & 0xf;
                let fg_color = (color_value >> 4) & 0xf;

                for i in 0..8 {
                    let char_on = char_value >> i != 0;
                    data[(x + i) + y * width] = if char_on { bg_color } else { fg_color };
                }
            }
        }

        data
    }

    pub fn load_image(&mut self, image: Vec<u8>) {
        if self.state.len() < image.len() {
            panic!(
                "image larger than state! - image: {}, state: {}",
                image.len(),
                self.state.len()
            )
        } else if self.state.len() > image.len() {
            panic!(
                "image smaller than state! - image: {}, state: {}",
                image.len(),
                self.state.len()
            )
        }

        self.state = image.into_iter().map(Some).collect()
    }
}
