use crate::*;

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        self();
        println!("test {} ... ok", core::any::type_name::<T>());
    }
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Testable]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    println!("test result: ok.");

    exit_success();
}

#[cfg(test)]
pub fn exit_success() -> ! {
    #[cfg(target_arch = "riscv64")]
    let qemu_exit_handle = qemu_exit::RISCV64::new(0x100000);
    #[cfg(target_arch = "aarch64")]
    let qemu_exit_handle = qemu_exit::AArch64::new();

    qemu_exit_handle.exit_success();
}

#[cfg(test)]
pub fn exit_failure() -> ! {
    #[cfg(target_arch = "riscv64")]
    let qemu_exit_handle = qemu_exit::RISCV64::new(0x100000);
    #[cfg(target_arch = "aarch64")]
    let qemu_exit_handle = qemu_exit::AArch64::new();

    qemu_exit_handle.exit_failure();
}

#[test_case]
fn test1() {
    assert_eq!(1, 1);
}

#[test_case]
fn test2() {
    fn fib(n: i32) -> i32 {
        if n < 2 {
            1
        } else {
            fib(n - 1) + fib(n - 2)
        }
    }
    assert_eq!(fib(10), 89);
}

#[test_case]
fn test3() {
    use alloc::vec::Vec;
    let mut a = Vec::new();
    a.push(2);
    a.push(3);
    assert_eq!(a.len(), 2);
    assert_eq!(a[0], 2);
    assert_eq!(a[1], 3);
}
