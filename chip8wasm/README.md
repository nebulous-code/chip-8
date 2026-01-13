# chip8wasm

This crate provides a `wasm-bindgen` wrapper around the CHIP-8 core emulator
from `chip8sys`. It exposes a web-friendly API for JavaScript and TypeScript
clients while keeping emulator logic inside the Rust core.

Repository:
https://github.com/nebulous-code/chip-8

## Install (crates.io)

```bash
cargo add chip8wasm
```

## Build for Web

Use `wasm-pack` to build the package for browser consumption.

```bash
wasm-pack build --target web --out-dir pkg --out-name chip8wasm
```

The output in `pkg/` is a standard ES module package that front ends can
consume via bundlers like Vite.

## JavaScript Usage

```js
import initWasm, { Chip8Wasm } from "chip8wasm";

async function run(romBytes) {
  await initWasm();
  const emulator = new Chip8Wasm();

  emulator.loadRom(romBytes);
  emulator.tick(10);
  emulator.tickTimers(1);

  const framebuffer = emulator.framebuffer();
  console.log(framebuffer.length);
}

run(romBytes);
```

## API Highlights

- `loadRom(romBytes)` loads a ROM byte buffer.
- `tick(cycles)` advances CPU cycles.
- `tickTimers(ticks)` advances delay/sound timers.
- `setKeys(mask)` sets keypad state via a bitmask.
- `framebuffer()` returns an unpacked pixel array.

## Related Crates

- `chip8sys`: The emulator core that `chip8wasm` wraps.

## Contributing

Fork the repository and open a pull request with the changes.
