#[no_mangle]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn start() {
    use crate::*;
    println!("PRESENT DAY\n  PRESENT TIME");
    loop {}
}
