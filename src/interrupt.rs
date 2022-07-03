use const_default::ConstDefault;

pub trait Backup: ConstDefault {
    fn save_and_off() -> Self
    where
        Self: Sized;
    fn restore(&self);
}

#[derive(ConstDefault)]
pub struct DummyBackup;

impl DummyBackup {
    pub const fn new() -> Self {
        Self
    }
}

impl Backup for DummyBackup {
    fn save_and_off() -> Self {
        Self
    }

    fn restore(&self) {}
}

#[cfg(target_arch = "riscv64")]
pub type ArchInterruptFlag = InterruptFlag<crate::arch::riscv64::riscv::InterruptFlag>;
#[cfg(target_arch = "aarch64")]
pub type ArchInterruptFlag = InterruptFlag<DummyBackup>;
#[cfg(target_arch = "x86_64")]
pub type ArchInterruptFlag = InterruptFlag<DummyBackup>;

#[derive(ConstDefault)]
pub struct InterruptFlag<T: Backup> {
    inner: T,
}

impl<T: Backup> Backup for InterruptFlag<T> {
    fn save_and_off() -> Self {
        Self {
            inner: T::save_and_off(),
        }
    }

    fn restore(&self) {
        self.inner.restore()
    }
}
