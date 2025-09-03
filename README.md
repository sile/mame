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

### Input Binding System
- **Mouse and Keyboard Support**: Handle both keyboard events (keys, modifiers) and mouse events (clicks, scrolling, dragging)
- **Configurable Input Bindings**: Define custom input bindings with JSON/JSONC configuration files using the `triggers` field
- **Context-Aware Bindings**: Support for multiple input contexts with different binding sets that can be switched at runtime
- **Input Pattern Matching**: Flexible input parsing supporting:
  - Keyboard: modifiers (`C-c`, `M-x`), special keys (`<UP>`, `<ENTER>`), printable characters, and hex notation (`0x7f`)
  - Mouse: clicks (`<LEFTCLICK>`, `<RIGHTCLICK>`), scrolling (`<SCROLLUP>`, `<SCROLLDOWN>`), and dragging (`<DRAG>`)
- **Binding Tracking**: Track the last processed input and successfully matched binding with unique identifiers

### Terminal Utilities
- **Unicode-Aware Rendering**: Proper handling of wide characters (CJK, emojis) and zero-width characters
- **Terminal Frames**: Built on [tuinix](https://crates.io/crates/tuinix) with Unicode width estimation
- **Column Calculation**: Accurate display width calculation for international text

### UI Components
- **Legend Rendering**: Create bordered legend boxes for displaying input binding help with automatic sizing
- **File Preview**: Side-by-side file preview panes with automatic layout

### Command Execution
- **External Commands**: Execute system commands with configurable stdin/stdout/stderr handling

### Configuration
- **JSONC Support**: JSON with comments for human-friendly configuration files
- **Variable Resolution**: Template variables with environment variable support
- **Binding Structure**: Updated configuration format using `bindings` instead of `keybindings`, with `triggers` arrays for each binding

### Formatting Utilities
- **Input Display**: Format both keyboard and mouse inputs for display purposes
- **Flexible Padding**: Create padded strings with customizable characters

