#[macro_export]
macro_rules! print {
    ($($args:tt)+) => {{
		#[cfg(any(target_arch = "riscv64", target_arch = "x86_64"))]
		#[cfg(target_board = "virt")]
		#[allow(unused_unsafe)]
        {
            use core::fmt::Write;
            let _ = unsafe { write!(crate::device::common::uart::UART, $($args)+) };
        }

		#[cfg(target_arch = "aarch64")]
		#[cfg(target_board = "raspi3b")]
		#[allow(unused_unsafe)]
        {
            use core::fmt::Write;
            let _ = unsafe { write!(crate::device::raspi3b::uart::UART, $($args)+) };
        }
    }};
}

#[macro_export]
macro_rules! println
{
	() => ({
		print!("\r\n")
	});
	($fmt:expr) => ({
		print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		print!(concat!($fmt, "\r\n"), $($args)+)
	});
}
