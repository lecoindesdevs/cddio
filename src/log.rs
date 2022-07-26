use log::{Record, Level, Metadata, LevelFilter, SetLoggerError};
#[macro_use]
pub mod macros {
    #[doc(alias = "log::error")]
    #[macro_export]
    macro_rules! log_error {
        ($($arg:tt)*) => {
            log::error!(target:"cddio", $($arg)*)
        };
    }
    #[doc(alias = "log::warn")]
    #[macro_export]
    macro_rules! log_warn {
        ($($arg:tt)*) => {
            log::warn!(target:"cddio", $($arg)*)
        };
    }
    #[doc(alias = "log::info")]
    #[macro_export]
    macro_rules! log_info {
        ($($arg:tt)*) => {
            log::info!(target:"cddio", $($arg)*)
        };
    }
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    #[inline]
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) && record.target() == "cddio" {
            println!("[{}] {}", record.level(), record.args());
        }
    }
    #[inline]
    fn flush(&self) {}
}


static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|_| log::set_max_level( if cfg!(debug_assertions) {LevelFilter::Trace} else {LevelFilter::Warn} ))
}
