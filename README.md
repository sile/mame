mame
====

[![mame](https://img.shields.io/crates/v/mame.svg)](https://crates.io/crates/mame)
[![Documentation](https://docs.rs/mame/badge.svg)](https://docs.rs/mame)
[![Actions Status](https://github.com/sile/mame/workflows/CI/badge.svg)](https://github.com/sile/mame/actions)
![License](https://img.shields.io/crates/l/mame)

This library provides common building blocks to build TUI (Terminal User Interface) applications.

"mame (豆)" means "bean" in Japanese.
Just like beans are versatile ingredients that can be transformed into countless dishes - from miso soup to tofu, from coffee to chocolate -
this library provides flexible, composable components that can be combined to create diverse TUI applications.

## Features

### Action System
- **Configurable Actions**: Define custom actions with JSON/JSONC configuration files
- **Context-Aware Keybindings**: Support for multiple input contexts with different keybinding sets
- **Key Matching**: Flexible key input parsing supporting modifiers (`C-c`, `M-x`), special keys (`<UP>`, `<ENTER>`), and printable characters

### Terminal Utilities
- **Unicode-Aware Rendering**: Proper handling of wide characters (CJK, emojis) and zero-width characters
- **Terminal Frames**: Built on [tuinix](https://crates.io/crates/tuinix) with Unicode width estimation
- **Column Calculation**: Accurate display width calculation for international text

### UI Components
- **Legend Rendering**: Create bordered legend boxes for displaying keybinding help
- **File Preview**: Side-by-side file preview panes with automatic layout

### Command Execution
- **External Commands**: Execute system commands with configurable stdin/stdout/stderr handling

### Configuration
- **JSONC Support**: JSON with comments for human-friendly configuration files
- **Variable Resolution**: Template variables with environment variable support

