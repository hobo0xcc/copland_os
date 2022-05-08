use crate::*;
use log::{Level, LevelFilter, Metadata, Record};

static LOGGER: KernelLogger = KernelLogger {};

pub struct KernelLogger {}

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "[{}:{}] {} - {}",
                record.file().unwrap(),
                record.line().unwrap(),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

pub fn init_logger() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .unwrap();
}
