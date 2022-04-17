#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate alloc;
extern crate hashbrown;
extern crate lazy_static;
extern crate spin;
extern crate volatile;

pub mod allocator;
pub mod arch;
pub mod device;
pub mod kernlock;
pub mod print;
