---
title: Chip8 Emulator
section: 1
header: User Manual
footer: chip8 0.1.0
date: April 3, 2024
---

# NAME

chip8 - Chip8 Emulator that can emulate, disassemble and debug ROMs.

# SYNOPSIS

**chip8** [*OPTIONS*] [*ROM*]

**chip8** emu [*ROM*]

**chip8** dis [*ROM*]

**chip8** dbg [*ROM*]

# DESCRIPTION

This Chip8 Emulator can Emulate, Disassemble and Debug ROMs.

For help on using the debugger, start the debugger and enter h.

# OPTIONS

**emu**
: emulates a give ROM

**dis**
: disassembles a given ROM

**dbg**
: starts the emulator in debugger mode for the given ROM

# EXAMPLES

**chip8 emu roms/test_opcode.ch8** Emulates roms/test_opcode.ch8.

**chip8 dis roms/test_opcode.ch8** Disassembles roms/test_opcode.ch8 and prints the memory address, the opcode and describes the behavior in words.

**chip8 dbg roms/test_opcode.ch8** Opens the emulator in debug mode.

# AUTHORS

Written by Conrad H. Carl.

# SEE ALSO

Github-Page: https://github.com/chc0815/chip8
