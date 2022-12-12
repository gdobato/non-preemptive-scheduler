# Non-preemptive scheduler

[![format](https://github.com/gdobato/non-preemptive-scheduler/actions//workflows/format.yml/badge.svg)](https://github.com/gdobato/non-preemptive-scheduler/actions/workflows/format.yml) 
[![lib](https://github.com/gdobato/non-preemptive-scheduler/actions//workflows/lib.yml/badge.svg)](https://github.com/gdobato/non-preemptive-scheduler/actions/workflows/lib.yml) 
[![examples](https://github.com/gdobato/non-preemptive-scheduler/actions/workflows/examples.yml/badge.svg)](https://github.com/gdobato/non-preemptive-scheduler/actions/workflows/examples.yml)

Basic non-preemptive scheduler to control task execution upon cycle completion and external events

Examples show its use running on a ARM Cortex-M4 MCU (STM32F429I-DISC1 board) along with some [rust-embedded crates](https://github.com/rust-embedded)

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
cargo build --example led_blinky
```
### Flash on target
```
cargo embed [--release] --example <example_name>
```
e.g :
```
cargo embed --example led_blinky
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
cargo embed --example led_blinky
```