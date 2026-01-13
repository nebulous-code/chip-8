# CHIP-8 in Rust

This repository contains the CHIP-8 emulator core and a WASM wrapper for web
front ends. It is designed for developers who want to embed a CHIP-8 emulator
inside native or browser-based projects.

## Demos

Live demo:
https://nebulous-code.github.io/chip8-vue/

### Vue Online Demo

![vue_web_wrapper](.../assets/screen_shots/CHIP-8_vue_wrapper.png)
Web app wrapper for the CHIP-8 library. 

### egui eframe Desktop App

![egui_eframe_wrapper](.../assets/screen_shots/CHIP-8_eframe_ibm_logo.png)

## About CHIP-8

Background and project notes live in [ABOUT.md](ABOUT.md).

## Workspace Overview

This workspace ships two crates:

- `chip8sys`: The emulator core, timing, input, and framebuffer logic.
- `chip8wasm`: A `wasm-bindgen` wrapper around `chip8sys` for JavaScript/TypeScript.

`chip8wasm` is a thin wrapper that exposes a stable, web-friendly API while keeping all emulator behavior in `chip8sys`. That means any fixes or emulator accuracy improvements land once in the core and flow through both native and web clients.

## Install (crates.io)

Crates available through crates.io
[chip8sys](https://crates.io/crates/chip8sys)
[chip8wasm](https://crates.io/crates/chip8wasm)

```bash
cargo add chip8sys
cargo add chip8wasm
```

## Native Integration Example (chip8sys)

```rust
use chip8sys::chip8::Chip8Sys;

fn main() {
    let mut emulator = Chip8Sys::new_chip_8();
    let rom_bytes = std::fs::read("roms/1-chip8-logo.ch8")
        .expect("ROM should load");

    emulator.load_rom_bytes(&rom_bytes);

    if let Err(error) = emulator.tick(10) {
        println!("Chip-8 tick error: {:?}", error);
    }

    emulator.tick_timers(1);
    let framebuffer = emulator.framebuffer_packed();

    println!("Framebuffer bytes: {}", framebuffer.len());
}
```

## Web Integration Example (chip8wasm)

```js
import initWasm, { Chip8Wasm } from "chip8wasm";

async function run() {
  await initWasm();
  const emulator = new Chip8Wasm();

  // romBytes should be a Uint8Array fetched or loaded from a file input.
  emulator.loadRom(romBytes);
  emulator.tick(10);
  emulator.tickTimers(1);

  const framebuffer = emulator.framebuffer();
  console.log(framebuffer.length);
}

run();
```

## Integrating Into Your Project

Typical embedding flow:

1. Load ROM bytes and call `load_rom_bytes` (native) or `loadRom` (WASM).
2. Update keypad state with `set_keys` / `setKeys` each frame.
3. Advance CPU cycles with `tick`, then advance timers with `tick_timers`.
4. Read the framebuffer and render it to a pixel surface.

For a working web integration, see the `chip8-vue` project [in this repo](https://github.com/nebulous-code/chip8-vue). 
For a native UI example, see the `chip8eframe` project [in this repo](https://github.com/nebulous-code/chip8eframe).

## Contributing

Contributions are welcome. Please fork the repository and open a pull request with the changes.

Project repository:
https://github.com/nebulous-code/chip-8
  - Clone the repository
  - `cargo build --release`
  - Navigate to Target Directory
  - Run CHIP-8

Open Issues can be found in [TODO.md](./TODO.md)

## About Me

This CHIP-8 Emulator is created by Nicholas Licalsi.

Find more of my projects at: [github.com/nebulous-code](https://github.com/nebulous-code)
