# What is a CHIP-8

CHIP-8 is a interpreted programming language for the COSMAC VIP. This means
that it was a program running on a computer that allowed users to more easily
write code and then immediately play or run. It was not a game console like an
Atari or Gameboy.

The original CHIP-8 was created by Joseph Weisbecker in the 1970s for his 1802
microprocessor. It was designed to make it easy to program games compared to
other languages that ran on systems at that time.

I took on this emulation project to better understand low level programming,
better understand older 8-bit systems, and to practice rust. The development of
this system was primarily done through test driven development. Originally with
tests I wrote myself based on documentation. Then through tests written by the
CHIP-8 community. See Accuracy, Limitations & Quirks below to learn more.

To get started click the Run button in the Control Flow window. You can
pause the CHIP-8 emulator at any time and step through the program one op code
at a time to get a sense of what the emulator is doing. If you need to reset
the emulator a button is provided.

Have fun and I hope you enjoy,

[Nicholas Licalsi](https://github.com/nebulous-code)

## Configuring This CHIP-8

There are a number of windows that can be opened or closed by clicking on them
in the right hand side.

If you'd like to get a sense of the registers, program counter, or other
internals of the CHIP-8 then you can view the Compute Info window.

If you'd like to shrink or expand the CHIP-8 display, configure colors, or
change from dark to light mode, the Screen Config window is available.

If you'd like to change ROMS you can switch through precompiled ROMS in the
ROMS window. Return to this About window if you want to see what the ROM that
is currently loaded does or how to control it. ROMS can not be uploaded at this
time but to see the ROMS content check the project's repository linked below.

Quirks can be configured and tested via the Quirks window. Learn more about
what quirks are in this emulator in the Quirks section below.

To better understand the controls of the CHIP-8 jump to the Controls section below.

## ROMS

A few ROMS have been preloaded onto this CHIP-8 and can be ran by selecting
them in the ROMS Window.

The majority of the ROMS are testing ROMS that I used during the development of
this project. Run them and see how well I did. You may understand them better
after reading the Limitations & Quirks section below.

I've uploaded a few creative commons games as well. You can control them using
the controls outlined below.

Specific information about the current ROM is displayed in the bottom section
of this window. It will have specific controls for the game (like movement,
selection, pausing, etc.)

## Chip 8 Controls

The original CHIP-8 had a 4 by 4 keyboard similar to a number pad with hex
digits A through F on the bottom and sides.

It was laid out like this:

| 1   | 2   | 3   | C   |
| --- | --- | --- | --- |
| 4   | 5   | 6   | D   |
| 7   | 8   | 9   | E   |
| A   | 0   | B   | F   |

This emulator maps the keys above to the first 4 number keys and then first
four keys of the 3 letter key rows on a QWERTY keyboard.

| 1   | 2   | 3   | 4   |
| --- | --- | --- | --- |
| Q   | W   | E   | R   |
| A   | S   | D   | F   |
| Z   | X   | C   | V   |

CHIP-8 input can be delayed because it did not employ an interrupt to stop
computing and take into account the keys being pressed like modern systems.

Instead there is a wait for key press operation code and a pole keys being
pressed op code. Because of this your input may feel delayed by modern
standards

## Accuracy, Limitations & Quirks

### Accuracy

When emulating a console it is important to know that it's being done right.
However, I don't have a system that runs CHIP-8 and have never played a CHIP-8
on original hardware.

Because of this I depended on members of the emulator and CHIP-8 community. Any
success with this project was thanks to their documentation efforts, any errors
are my own.

I gained familiarity with the CHIP-8 system through John Earnest's:
[OCTO Emulator](https://johnearnest.github.io/Octo/)

I developed this CHIP-8 emulator using the tests in:
[Timendus's Test Suite](https://github.com/Timendus/chip8-test-suite).

Most of the documentation and specs I used were found in:
[Tobaisvl's Awesome CHIP-8 Repository](https://github.com/tobiasvl/awesome-chip-8)

Creative Commons ROMS were found at:
[John Earnest's CHIP-8 Archive](https://johnearnest.github.io/chip8Archive/)

Original creators have been credited when available via the About this ROM
section of this window.

This CHIP-8 emulator library was written purely in rust.

The presentation of the emulator was done with egui using eframe to render WASM
for the web. [egui by Emilk](github.com/emilk/egui)

### Limitations

This emulator aims to specifically emulate the earliest CHIP-8. Including its
quirks. Future versions of the CHIP-8 like the SUPER-CHIP and XO-CHIP have
their own quirks and their own advancements.

I've implemented some of those quirks where convenient, but to limit the scope
of this project advanced versions of the CHIP-8 have not been accurately
emulated at this time.

ROMS written for those versions of the CHIP-8 may not "feel" right if they are
loaded here. Some operation codes will not execute which could lead to hanging,
memory corruption, or other unintended CHIP-8 issues. However, the emulator
itself will not crash or panic if bad op codes are submitted.

### Quirks

Quirks are unintended limitations or consequences of the real life
implementation of the CHIP-8 design. These are either oversights that the
original developers might have made or results of using the original hardware
in a way that the original developers didn't intend.

I've intentionally implemented 4 quirks in this design:

#### Screen Clipping / Wrapping

When a sprite is drawn at the right hand or bottom of the screen the Original
CHIP-8 would clip the sprite at the edge making it look like the sprite was
disappearing out of frame.

I personally like the look of sprites wrapping around to the other side of the
screen. Future CHIP-8 systems also had wrapping on as default.

#### Register F Reset

On the original CHIP-8 system whenever the logical operation op code is used
(AND, OR, or XOR) the carry register (Register 0xF) was reset.

This seems to be an unintentional consequence of the original hardware which
was rectified in future CHIP-8 iterations.

Since AND, OR, and XOR don't have a carry bit I did not originally reset
Register F. When I realized I needed to implement it, I made it a configurable
quirk flag.

#### Modify Register X in Place

In the original CHIP-8 implementation the shift bits left and right commands
(0x8XY6 and 0x8XYE) took into account the Y place and shifted Register Y by one
bit left or right and then stored it in Register X.

In future implementations of CHIP-8 the 0x8XY6 and 0x8XYE commands did not take
into account Register Y. Instead they shifted the value stored in Register X
left or right and stored it back in Register X.

Because the original spec sheet I was reading for more advanced CHIP-8 systems
I had to add taking Register Y into account and left the functionality in as a
configurable quirk.

#### Increment Index

The CHIP-8 has the ability to store the registers into memory and read them out
of memory. Op Codes 0xFX55 and 0xFX65.

0xFX55 will store register 0 to register X into memory starting at the location
currently stored in register I.

0xFX65 will read register 0 to register X out of memory starting at register I.

The original implementation of CHIP-8 incremented the value of the I register
as it did this work (due to how the hardware incremented through memory to
store these registers). However, future implementations did not increment
register I.

I did not originally setup register I to increment. And once again, when I
found myself needing to add that quirk I made it configurable.

## Future Enhancements

There are a number of enhancements that I would love to make to this CHIP-8
emulator given more time and resources.

Those enhancements can be found in the [TODO.md](TODO.md) file in this repository.

They include but are not limited to:

- [ ] Enable Users to Upload their own ROMS
- [ ] Improve Delay Timer and Sound Timer
- [ ] Emulate more advanced CHIP-8 systems

## Running This Emulator

Find the code repository of this project at:
[Nebulous Code CHIP-8 Repository](https://github.com/nebulous-code/chip-8)

### TODO: Make this section accurate

The CHIP-8 Library can be imported as a rust crate.
More info at crates.io: TBD
View a web demo of this project at: TBD
Download a local version to run on your own machine here:

- Windows:
- MacOS:
- Linux:
- Others:
  - Install cargo via rustup.org
  - Clone the repository
  - `cargo build --release`
  - Navigate to Target Directory
  - Run CHIP-8

## About Me

This CHIP-8 Emulator is created by Nicholas Licalsi.

Find more of my projects at: [github.com/nebulous-code](https://github.com/nebulous-code)
