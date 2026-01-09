#![crate_name = "chip8wasm"]
#![crate_type = "cdylib"]

//! This crate exposes the Chip-8 emulator through a WebAssembly-friendly API.

// Edits require a rebuild `wasm-pack build chip8wasm --target web`

use wasm_bindgen::prelude::*;

use chip8sys::chip8::{Chip8Sys, DISPLAY_HEIGHT, DISPLAY_PIXELS, DISPLAY_WIDTH, TimerMode};
use chip8sys::chip8error::Chip8Error;

/// This struct wraps the Chip-8 emulator for JavaScript callers.
#[wasm_bindgen]
pub struct Chip8Wasm {
    emulator: Chip8Sys,
}

#[wasm_bindgen]
impl Chip8Wasm {
    /// This function constructs a new Chip-8 emulator instance.
    /// Arguments: none.
    /// Returns: A new Chip-8 WASM wrapper.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Chip8Wasm {
        let mut emulator = Chip8Sys::new_chip_8();
        emulator.set_timer_mode(TimerMode::External);
        Chip8Wasm { emulator }
    }

    /// This function resets the emulator to its initial state.
    /// Arguments: none.
    /// Returns: none.
    #[wasm_bindgen(js_name = "reset")]
    pub fn reset(&mut self) {
        self.emulator.reset();
    }

    /// This function loads a ROM byte buffer into memory.
    /// Arguments:
    /// - rom_bytes: The ROM bytes to load.
    /// Returns: none.
    #[wasm_bindgen(js_name = "loadRom")]
    pub fn load_rom(&mut self, rom_bytes: &[u8]) {
        self.emulator.load_rom_bytes(rom_bytes);
    }

    /// This function updates the keypad state using a 16-bit bitmask.
    /// Arguments:
    /// - mask: The bitmask of pressed keys.
    /// Returns: none.
    #[wasm_bindgen(js_name = "setKeys")]
    pub fn set_keys(&mut self, mask: u16) {
        self.emulator.set_keys_mask(mask);
    }

    /// This function advances the emulator by a number of CPU cycles.
    /// Arguments:
    /// - cycles: The number of cycles to execute.
    /// Returns: Ok on success, otherwise a JS error.
    #[wasm_bindgen(js_name = "tick")]
    pub fn tick(&mut self, cycles: u32) -> Result<(), JsValue> {
        self.emulator.tick(cycles).map_err(to_js_error)
    }

    /// This function advances the delay and sound timers by a number of ticks.
    /// Arguments:
    /// - ticks: The number of 60Hz timer ticks to apply.
    /// Returns: none.
    #[wasm_bindgen(js_name = "tickTimers")]
    pub fn tick_timers(&mut self, ticks: u32) {
        self.emulator.tick_timers(ticks);
    }

    /// This function returns the framebuffer as a 0/1 byte array.
    /// Arguments: none.
    /// Returns: A framebuffer array with one byte per pixel.
    #[wasm_bindgen(js_name = "framebuffer")]
    pub fn framebuffer(&self) -> Vec<u8> {
        unpack_framebuffer(self.emulator.framebuffer_packed())
    }

    /// This function returns the packed framebuffer bytes.
    /// Arguments: none.
    /// Returns: The packed framebuffer bytes.
    #[wasm_bindgen(js_name = "framebufferPacked")]
    pub fn framebuffer_packed(&self) -> Vec<u8> {
        self.emulator.framebuffer_packed().to_vec()
    }

    /// This function returns the current delay timer value.
    /// Arguments: none.
    /// Returns: The delay timer value.
    #[wasm_bindgen(js_name = "delayTimer")]
    pub fn delay_timer(&self) -> u8 {
        self.emulator.delay_timer()
    }

    /// This function returns the current sound timer value.
    /// Arguments: none.
    /// Returns: The sound timer value.
    #[wasm_bindgen(js_name = "soundTimer")]
    pub fn sound_timer(&self) -> u8 {
        self.emulator.sound_timer()
    }

    /// This function reports whether the sound timer is active.
    /// Arguments: none.
    /// Returns: True if sound is playing.
    #[wasm_bindgen(js_name = "isSoundPlaying")]
    pub fn is_sound_playing(&self) -> bool {
        self.emulator.is_sound_playing()
    }
}

/// This function returns the Chip-8 display width in pixels.
/// Arguments: none.
/// Returns: The display width.
#[wasm_bindgen(js_name = "displayWidth")]
pub fn display_width() -> u32 {
    DISPLAY_WIDTH as u32
}

/// This function returns the Chip-8 display height in pixels.
/// Arguments: none.
/// Returns: The display height.
#[wasm_bindgen(js_name = "displayHeight")]
pub fn display_height() -> u32 {
    DISPLAY_HEIGHT as u32
}

/// This function returns the Chip-8 display pixel count.
/// Arguments: none.
/// Returns: The display pixel count.
#[wasm_bindgen(js_name = "displayPixels")]
pub fn display_pixels() -> u32 {
    DISPLAY_PIXELS as u32
}

/// This function converts packed framebuffer bytes into a 0/1 pixel array.
/// Arguments:
/// - packed: The packed framebuffer bytes.
/// Returns: The unpacked framebuffer.
fn unpack_framebuffer(packed: &[u8]) -> Vec<u8> {
    let mut output = vec![0u8; DISPLAY_PIXELS];
    for (byte_index, byte) in packed.iter().enumerate() {
        let base = byte_index * 8;
        if base >= DISPLAY_PIXELS {
            break;
        }
        for bit in 0..8u32 {
            let pixel_index = base + bit as usize;
            if pixel_index >= DISPLAY_PIXELS {
                break;
            }
            let mask = 0b1000_0000u8 >> bit;
            output[pixel_index] = if byte & mask == mask { 1 } else { 0 };
        }
    }
    output
}

/// This function converts a Chip-8 error into a JS error value.
/// Arguments:
/// - error: The chip-8 error to convert.
/// Returns: The JS error value.
fn to_js_error(error: Chip8Error) -> JsValue {
    JsValue::from_str(&format!("{error:?}"))
}
