pub const EXCEPTION_SYNC: usize = 0;
pub const EXCEPTION_IRQ: usize = 1;
pub const EXCEPTION_FIQ: usize = 2;
pub const EXCEPTION_SERROR: usize = 3;

#[no_mangle]
pub unsafe extern "C" fn kernel_exception(
    exception_type: usize,
    exception_cause: usize,
    return_address: usize,
    saved_pstate: usize,
    fault_address: usize,
) -> ! {
    panic!(
        "type: {}, exception-class: {:#b}, ret: {:#x}, pstate: {:#x}, fault-addr: {:#x}",
        exception_type,
        (exception_cause >> 26) & 0x3f,
        return_address,
        saved_pstate,
        fault_address
    );
}
