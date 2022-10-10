# Hello-rust

Small project based on [cortex-m-quickstart template](https://github.com/rust-embedded/cortex-m-quickstart) to run Rust on a Cortex-M4F target (STM32F429I-DISC1 board)

## Installation (Unix-like OS)
**Toolchain**
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
**Suport for ARM-CM targets**
```
rustup target add thumbv6m-none-eabi thumbv7m-none-eabi thumbv7em-none-eabi thumbv7em-none-eabihf
```

## Build
**Unoptimized**
```
cargo build
```
**Optimized**
```
cargo build --release
```