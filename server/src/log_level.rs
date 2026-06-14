/// Hierarchical log levels for the HoM logging system.
///
/// Controlled by the `hoi4.logLevel` setting in VS Code.
/// Each level includes all levels below it:
///   ERROR (0) → critical failures only
///   WARN  (1) → warnings + errors
///   INFO  (2) → scan results, config, progress — DEFAULT
///   DEBUG (3) → per-file validation timing, scanner details
///   TRACE (4) → every LSP event (didOpen, didChange, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl LogLevel {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => LogLevel::Error,
            1 => LogLevel::Warn,
            2 => LogLevel::Info,
            3 => LogLevel::Debug,
            4 => LogLevel::Trace,
            _ => LogLevel::Info,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn prefix(self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}
