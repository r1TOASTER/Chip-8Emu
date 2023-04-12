#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PixelState {
    On,
    Off,
}
pub struct Screen {
    pixels: [[PixelState; 64]; 32],
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            pixels: [[PixelState::Off; 64]; 32],
        }
    }

    pub fn display_pixels(&self) {
        for pixel_row in self.pixels {
            for pixel in pixel_row {
                match pixel {
                    PixelState::Off => print!("⬛"),
                    PixelState::On => print!("⬜"),
                }
            }
            println!("");
        }
    }

    pub fn clear_screen(&mut self) {
        for pixel_row in self.pixels {
            for mut _pixel in pixel_row {
                _pixel = PixelState::Off;
            }
        }
    }

    pub fn get_pixel(&self, x: &u8, y: &u8) -> Option<PixelState> {
        self.pixels.get(*y as usize)?.get(*x as usize).copied()
    }

    pub fn set_pixel(&mut self, x: &u8, y: &u8, value: PixelState) {
        self.pixels[*y as usize][*x as usize] = value
    }
}