# Copland OS

This project is a work in progress. By the way, *Have you ever seen the lain?*

# Requirements

- rust toolchain (nightly)
  - [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
  - `rustup toolchain install nightly && rustup default nightly`
- qemu
  - [https://www.qemu.org/download/](https://www.qemu.org/download/)

# Build

## Debug build

```bash
cargo build
```

## Release build

```bash
cargo build --release
```

The compiled executable is `./target/riscv64gc-unknown-none-elf/[build mode]/copland_os`

# Run

```bash
cargo run # this requires qemu-system-riscv64
```

# Debug with gdb

```bash
./tools/debug_[board name].sh
```

```bash
gdb -x tools/script_[board name].gdb
```

# Supported boards

## riscv64gc-unknown-none-elf

- virt (wip)

## aarch64-unknown-none-softfloat

- raspi3b (wip)