# Gesha

## Cross-compiling for Pi Zero

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
