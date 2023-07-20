# Rust Chip-8 Emulator

This is a Chip-8 emulator written in Rust. Chip-8 is an interpreted programming language that was first used in the mid-1970s. It was initially used on 8-bit microcomputers, but its simplicity makes it a popular choice for creating simple emulators.

## Description

The Chip-8 emulator interprets and executes Chip-8 programs. The Chip-8 system is capable of rendering graphics on a screen and receiving user input, making it perfect for games. Some of the classic games that have been written in Chip-8 include Pong, Space Invaders, and Tetris.

## Installation

1. Install [Rust](https://www.rust-lang.org/tools/install) and [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) if you haven't already.

2. Clone this repository:
    ```
    git clone https://github.com/GKaszewski/chip8.git
    ```

3. Build the emulator:
    ```
    cd chip8
    cargo build --release
    ```

## Usage

To run a Chip-8 program, you will need a Chip-8 ROM. Once you have a ROM, you can run it with the emulator like so:
```
./target/release/chip8 <path-to-rom>
```

## Notes
Even though it passes cortexm0's test suite, this emulator is not perfect and does not pass flags test :/ (no idea why tho). There are still some bugs that need to be fixed. If you find any bugs, please open an issue.
