# A CHIP-8 implementation in rust

This project is a WIP.


# Crates

This project contains two crates


## chip8_core

[chip8_core](chip8_core/) is a `no_std` implementation of the CHIP-8 core logic, including traits required to implement a CHIP-8 emulator


## chip8_tools

[chip8_tools](chip8_tools/) is a desktop (for now, linux only) implementation of a CHIP-8 emulator, based on `chip8_core`