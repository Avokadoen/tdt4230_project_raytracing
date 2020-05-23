use std::vec::Vec;
use std::option::Option;

use sdl2::keyboard::Keycode;

pub struct InputHandler {
    pub active_keys: Vec<Keycode>,
}

impl Default for InputHandler {
    fn default() -> InputHandler {
        Self {
            // preallocate some keys, probably wont be above 1o
            active_keys: Vec::with_capacity(10),
        }
    }
}

impl InputHandler {
    pub fn on_key_down(&mut self, key_o: Option<Keycode>) {
        // TODO: consider macro if we have to duplicate this more
        let key =  match key_o {
            Some(k) => k,
            None => return,
        };

        self.active_keys.push(key);
    }

    pub fn on_key_up(&mut self, key_o: Option<Keycode>) {
        let key =  match key_o {
            Some(k) => k,
            None => return,
        };

        match self.active_keys.iter().position(|k| k == &key) {
            Some(n) => { self.active_keys.swap_remove(n); },
            None => (),
        };
    }
}