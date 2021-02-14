# CHIP-8 Emulator/Interpreter

## Learning Material

Fortunately there is a lot of information about how to build a chip 8.

This was the main source of information to understand and get started on some of the opcodes:

- [This guide](www.multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/) as a guide to start

Others worth mentioning:

- [Cowgod's Chip8 Tech reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [Awesome Chip 8](https://chip-8.github.io/links/)

Some times I cheated and looked at some implementations both in C and in Rust:

- https://github.com/AlexEne/rust-chip8
- https://github.com/cookerlyk/Chip8

## Up and running

The repo is structured to enable multiple frontends to work with a core chip8 implementation. 

### SDL2

#### Requirements:

- Sdl2
- Rust

#### Running it:

After cloning the repo do:

`cargo run -- -r[om] <rom-name>`

Some roms might need adjusting how fast the cpu runs, you can do this using the `-h[ertz]` flag. By default, it runs @ 500hz.

#### Known limitations

- I'm yet to find a rom that blocks the execution until you press a key so that is not tested
- It is somewhat slow at times when rendering. I probably should try to optimize this at some point.

### Wasm

- TBD

### Tests

Run all the tests with `cargo t[est]`
