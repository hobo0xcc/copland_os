#!/bin/sh

cargo build
qemu-system-riscv64 -s -S -machine virt -bios none -m 256M -smp 1 -serial stdio -kernel ./target/riscv64gc-unknown-none-elf/debug/copland_os