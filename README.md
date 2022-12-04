# Hello-rust

[![format](https://github.com/gdobato/hello-rust/actions//workflows/format.yml/badge.svg)](https://github.com/gdobato/hello-rust/actions/workflows/format.yml) 
[![lib](https://github.com/gdobato/hello-rust/actions//workflows/lib.yml/badge.svg)](https://github.com/gdobato/hello-rust/actions/workflows/lib.yml) 
[![examples](https://github.com/gdobato/hello-rust/actions/workflows/examples.yml/badge.svg)](https://github.com/gdobato/hello-rust/actions/workflows/examples.yml)

Small project which makes use of [rust-embedded-wg](https://github.com/rust-embedded/wg) crates and references to run Rust on a Cortex-M4F target (STM32F429I-DISC1 board)

### Installation (Unix-like OS)
Toolchain
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

rustup target add thumbv6m-none-eabi thumbv7m-none-eabi thumbv7em-none-eabi thumbv7em-none-eabihf
```
[Cargo-embed](https://github.com/probe-rs/cargo-embed)
```
cargo install cargo-embed
```

### Build

```
cargo build [--release] --example <example_name>
```
e.g :
```
cargo build --example usb_dev_cdc
```
### Flash on target
```
cargo embed [--release] --example <example_name>
```
e.g :
```
cargo embed --example usb_dev_cdc
```

### Attach to target

Disable flash in Embed.toml of root directory

```
[default.flashing]
enabled = false
```
Run cargo embed
```
cargo embed [--release] --example <example_name>
```
e.g :
```
cargo embed --example usb_dev_cdc
```