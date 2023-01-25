# Emu8
A Chip-8 interpreter written in Rust.

![demo gif](docs/demo.gif)

# System
The [Chip-8](https://en.wikipedia.org/wiki/CHIP-8) is a virtual machine developed in the 1970's.
It supports 35 opcodes, has a 64x32 pixel display, and is capable of producing a single beep tone.
The use of two different opcodes to clear (`00E0`) the screen and draw (`Dxyn`) it means that games are inherently flickery.

See [the chip8 struct's docstring](src/chip8.rs) for more system details.

# Aspirations
 - [x] decoupled input, logic, and rendering
 - [x] fully tested opcodes
 - [x] fast forward
 - [x] rewind

# Running
```bash
cargo run -- run --file ~/path/to/file.ch8
```

The Chip-8 supports ROMS up to 3584 bytes in length (4K memory - 512 bytes for internal use).

Dmatlack has a [repository of games](https://github.com/dmatlack/chip8/tree/master/roms/games) that this emulator has been validated against.

# References
 - [Columbia University's Chip8 Design Specification](http://www.cs.columbia.edu/~sedwards/classes/2016/4840-spring/designs/Chip8.pdf)
 - [Cowgod's Chip-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
 - [Mastering Chip-8](http://mattmik.com/files/chip8/mastering/chip8.html)
 - [Craig Thomas's Chip8 Assembler](https://github.com/craigthomas/Chip8Assembler) for opcode mnemonics
