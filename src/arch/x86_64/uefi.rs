use uefi::prelude::*;
use uefi::Status;

extern crate alloc;

#[entry]
fn efi_main(_image: Handle, mut st: SystemTable<Boot>) -> Status {
    use core::fmt::Write;
    writeln!(st.stdout(), "Hello, world!").unwrap();
    loop {}
}
