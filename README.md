# Copland OS

This project is a work in progress. By the way, *Have you ever seen the lain?*

# Requirements

- Rust toolchain (nightly)
  - https://www.rust-lang.org/tools/install
  - `rustup toolchain install nightly && rustup default nightly`
- cargo-make
  - https://github.com/sagiegurari/cargo-make
  - `cargo install cargo-make`
- QEMU
  - https://www.qemu.org/download/
- dosfstools
  - macOS: `brew install dosfstools`
  - ubuntu: `apt install dosfstools`

# Build

## Debug build

```bash
# riscv64 virt
makers build-riscv64-dev

# aarch64 raspi3b
makers build-aarch64-dev
```

## Release build

```bash
# riscv64 virt
makers build-riscv64

# aarch64 raspi3b
makers build-aarch64
```

The compiled kernel binary is `./kernel.elf` .

# Run

```bash
# Debug mode
makers run-[riscv64 | aarch64]-dev # this requires QEMU

# Release mode
makers run-[riscv64 | aarch64] # this requires QEMU
```

# Debug with gdb

```bash
makers debug-[riscv64 | aarch64]-dev
```

```bash
rust-gdb -x tools/[virt | raspi3b].gdb
```

# Supported boards

## riscv64

- virt (wip)

## aarch64

- raspi3b (wip)