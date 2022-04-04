#!/bin/sh

cargo build --target=aarch64-unknown-none-softfloat
qemu-system-aarch64 -s -S -machine raspi3b -m 1G -kernel ./target/aarch64-unknown-none-softfloat/debug/copland_os