use core::arch::asm;

pub fn wait_msec(n: u64) {
    let (mut f, mut t, mut r): (u64, u64, u64);
    unsafe {
        asm!("mrs {}, cntfrq_el0", out(reg)f);
        asm!("mrs {}, cntpct_el0", out(reg)t);
        t += ((f / 1000).wrapping_mul(n)) / 1000;
        asm!("mrs {}, cntpct_el0", out(reg)r);
        while r < t {
            asm!("mrs {}, cntpct_el0", out(reg)r);
        }
    }
}

pub fn wait_cycles(mut n: u32) {
    if n != 0 {
        while n != 0 {
            n -= 1;
            unsafe {
                asm!("nop");
            }
        }
    }
}
