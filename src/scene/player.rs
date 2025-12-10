use std::collections::HashMap;

use winit::keyboard::KeyCode;

pub struct Player {
    pub is_key_pressed: HashMap<KeyCode, bool>,
    // add pos, rot, etc.
    // PlayerUniform (look at CameraUniform), that computes CTM based on pos, rot, etc.
}

const KEYS: [KeyCode; 4] = [KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyD, KeyCode::KeyS];

impl Player {
    pub fn new() -> Self {
        let mut keys = HashMap::new();
        for key in KEYS {
            keys.insert(key, false);
        }

        Self {
            is_key_pressed: keys,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, is_pressed: bool) {}

    pub fn update(&mut self) {
        // update pos/rot

        // call func to update uniform ctm
    }
}
