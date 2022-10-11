# Hello-rust

Small project based on [cortex-m-quickstart template](https://github.com/rust-embedded/cortex-m-quickstart) to run Rust on a Cortex-M4F target (STM32F429I-DISC1 board)

## Installation (Unix-like OS)
Toolchain
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Suport for ARM-CM targets
```
rustup target add thumbv6m-none-eabi thumbv7m-none-eabi thumbv7em-none-eabi thumbv7em-none-eabihf
```

[Cargo-flash](https://github.com/probe-rs/cargo-flash)
```
cargo install cargo-flash
```

[Cargo-runner](https://github.com/knurling-rs/probe-run)
```
cargo install probe-run
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

## Flash on target
```
cargo flash --chip STM32F429ZITx [--release]
```

## Launch runner
```
cargo run [--release]
```