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

# Build

## Debug build

```bash
makers build-[arch_name]-dev
```

## Release build

```bash
makers build-[arch_name]
```

The compiled executable is `./kernel.elf` .

# Run

```bash
# Debug mode
makers run-[arch_name]-dev # this requires QEMU

# Release mode
makers run-[arch_name] # this requires QEMU
```

# Debug with gdb

```bash
makers debug-[arch_name]
```

```bash
rust-gdb -x tools/[board_name].gdb
```

# Supported boards

## riscv64

- virt (wip)

## aarch64

- raspi3b (wip)