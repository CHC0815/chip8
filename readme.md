# Chip 8 Emulator

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#requirements">Requirements</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roms">ROMs</a></li>
    <li><a href="#manpage">MAN Page</a></li>
  </ol>
</details>

## About the Project

This Chip8 Emulator is a hobby project to learn more about emulators.
It's written in [Rust](https://www.rust-lang.org/) and uses [SDL2](https://www.libsdl.org/) for rendering and user input.

It's still WIP.

![Screenshot of the Chip8 Emulator](https://github.com/chc0815/chip8/blob/master/media/chip8.png?raw=true)

![Screenshot of the Chip8 Emulator running Fishie](https://github.com/chc0815/chip8/blob/master/media/fishie.png?raw=true)

## Getting Started

### Requirements

You need [rust](https://www.rust-lang.org/tools/install).

### Installation

```sh
git clone https://github.com/CHC0815/chip8.git
```

```sh
cd chip8
```

RUN:

```sh
cargo run -- (emu|dis) (path to rom)
```

BUILD:

```sh
cargo build --release
```

Output file: ./target/release/chip8

## Usage

### Emulate a ROM

```sh
chip8 emu rom.ch8
```

### Disassemble a ROM

```sh
chip8 dis rom.ch8
```

## ROMs

Most of the roms are from [https://github.com/Timendus/chip8-test-suite](https://github.com/Timendus/chip8-test-suite)

## MAN Page

```sh
man -l chip8.1
```
