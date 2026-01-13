# chip8sys

This crate contains the CHIP-8 emulator core. It handles CPU state, timers,
keypad input, and framebuffer output without any UI dependencies.

Repository:
https://github.com/nebulous-code/chip-8

## Install (crates.io)

```bash
cargo add chip8sys
```

## Quick Start

```rust
use chip8sys::chip8::Chip8Sys;

fn main() {
    let mut emulator = Chip8Sys::new_chip_8();
    let rom_bytes = std::fs::read("roms/1-chip8-logo.ch8")
        .expect("ROM should load");

    emulator.load_rom_bytes(&rom_bytes);
    emulator.tick(10).expect("tick should succeed");
    emulator.tick_timers(1);

    let framebuffer = emulator.framebuffer_packed();
    println!("Framebuffer bytes: {}", framebuffer.len());
}
```

## Integration Flow

Typical embedding loop:

1. Load ROM bytes and call `load_rom_bytes`.
2. Update keypad state via `set_keys` or `set_keys_mask`.
3. Advance CPU cycles using `tick`.
4. Advance timers using `tick_timers`.
5. Render the framebuffer output.

## Quirks and Configuration

Quirk flags are available through `Chip8Quirks`, and can be applied by calling
`set_quirks`. Timer behavior can be configured via `set_timer_mode`.

## Related Crates

- `chip8wasm`: A `wasm-bindgen` wrapper around `chip8sys` for browser use.

## Contributing

Fork the repository and open a pull request with the changes.
