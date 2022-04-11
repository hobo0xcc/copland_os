#[no_mangle]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn start() {
    loop {}
}
