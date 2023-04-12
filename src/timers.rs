use core::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};
pub struct DelayTimer {
    timer: u8,
}

pub struct SoundTimer {
    timer: u8,
}

pub trait Timer {
    fn get_timer(&self) -> u8;
    fn set_timer(&mut self, value: u8);
}

impl Timer for SoundTimer {
    fn get_timer(&self) -> u8 {
        self.timer
    }
    fn set_timer(&mut self, value: u8) {
        self.timer = value;
    }
}

impl SoundTimer {
    pub fn new() -> SoundTimer {
        SoundTimer { timer: u8::MAX }
    }
}

impl Timer for DelayTimer {
    fn get_timer(&self) -> u8 {
        self.timer
    }
    fn set_timer(&mut self, value: u8) {
        self.timer = value;
    }
}

impl DelayTimer {
    pub fn new() -> DelayTimer {
        DelayTimer { timer: u8::MAX }
    }
}

pub fn decrement_timer(timer_arc: Arc<Mutex<Box<dyn Timer + Send>>>) {
    loop {
        // Lock the mutex to access the timer
        let mut timer = timer_arc.lock().unwrap();

        // Decrement the timer value by 60
        let current_value = timer.get_timer();
        if current_value > 60 {
            timer.set_timer(current_value - 60);
        } else {
            timer.set_timer(0);
        }

        // Unlock the mutex to release the timer
        drop(timer);

        // Sleep for one second
        thread::sleep(Duration::from_secs(1));
    }
}