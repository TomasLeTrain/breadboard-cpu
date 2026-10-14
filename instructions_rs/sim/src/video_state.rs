//! Simulates physical video card

use sdl3::{
    gpu::Texture,
    pixels::{Color, Palette, PixelFormat},
    surface::Surface,
};

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
        let char_width = 40;
        let x_width = 8;
        let height = 240;
        let width = char_width * x_width;

        let mut data: Vec<u8> = std::iter::repeat_n(0, width * height * 4).collect::<Vec<_>>();

        let mut palette = Vec::new();

        for white in 0..=1 {
            for g in 0..=1 {
                for b in 0..=1 {
                    for r in 0..=1 {
                        palette.push(Color::RGB(
                            r * 127 + white * 65,
                            g * 127 + white * 65,
                            b * 127 + white * 65,
                        ));
                    }
                }
            }
        }

        for x in 0..char_width {
            for y in 0..height {
                let addr = (x + 1) | (y << 6);
                let color_value = self.memory()[addr].unwrap_or(0);
                let char_value = self.memory()[addr | (1 << 14)].unwrap_or(0);
                let bg_color = color_value & 0xf;
                let fg_color = (color_value >> 4) & 0xf;

                for i in 0..x_width {
                    let char_on = (char_value >> (x_width-i)) & 1 != 0;
                    let palette_data = if char_on { fg_color } else { bg_color };
                    let pixel = palette[palette_data as usize].to_u32(&PixelFormat::RGB332);
                    data[i + x * x_width + y * width] = pixel as u8;
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
