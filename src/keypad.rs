#[derive(Debug)]
pub struct Keypad {
    keys: [bool; 16],
}

impl Keypad {
    pub fn new() -> Keypad {
        Keypad { keys: [false; 16] }
    }

    pub fn press_key(&mut self, key: usize) {
        if key < 16 {
            self.keys[key] = true;
        }
    }

    pub fn release_key(&mut self, key: usize) {
        if key < 16 {
            self.keys[key] = false;
        }
    }

    pub fn is_pressed(&self, key: usize) -> bool {
        if key < 16 {
            self.keys[key]
        } else {
            false
        }
    }
}