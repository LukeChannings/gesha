# Gesha

![Gesha main tab](./docs/diagrams/gesha-main.png)

This repository contains the source code for the Gesha project - a platform for providing stable and consistent temperatures when brewing espresso.

The project contains the following:

- [`src/`](./src/) - the main application (Rust)
- [`ui/`](./ui/) / [`src-tauri`](./src-tauri/) - the UI source code (TypeScript)
- [`models`](./models/) - projects for modelling some aspect of the machine's behaviour (Python + SQL)
- [`docs/`](./docs/) - the source files for my project proposal and dissertation report (Rmarkdown / Typst)

## Usage

Make sure you have the following installed:

- Rust
- Just
- Python3 + Poetry
- Node.js + NPM

The project is managed with a [`Justfile`](./Justfile), run `just --list` for a list of recipes, or look at the Justfile.

The main Rust app cannot be run on devices that aren't the Raspberry Pi because of the `rppal` dependency. Ideally this module should be stubbed for local compiling and testing. For now the native compile target is `arm-unknown-linux-gnueabihf`.

## Setup

### macOS

Ensure both Homebrew and rustup are installed, then install the cross-compilation toolchain for ARMv7:

```
brew install armv7-unknown-linux-gnueabihf
rustup target add arm-unknown-linux-gnueabihf
```

Then compile gesha: `cargo build --target arm-unknown-linux-gnueabihf`

### Cross

If you're not on macOS, [cross-rs](https://github.com/cross-rs/cross) can be used to compile Gesha for the Raspberry Pi Zero from any OS, since it uses Docker containers (Linux) for the toolchain.

The Pi Zero v1 has a 32-bit ARMv6 CPU, the rust target is `arm-unknown-linux-gnueabihf` (See [Supported Targets](https://github.com/cross-rs/cross#supported-targets) - this target requires glibc 2.31).

The project can be compiled with:

```
cross build --target arm-unknown-linux-gnueabihf
```
