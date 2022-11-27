//! Configuration related definitions and implementation

use crate::{paint::ColorFormat, sculpt::RecordFormat, theme::Spectral, theme::Theme};
use log::LevelFilter;

/// Main configuration for a `SwingLogger`
pub struct Config {
    /// log level filter (logs below this severity will be ignored)
    pub level: LevelFilter,
    /// record formatting mode (determines how log records are structurally formatted)
    pub record_format: RecordFormat,
    /// color formatting mode (determines how log records are colored)
    pub color_format: Option<ColorFormat>,
    /// color theme (determines the color palette used to color log records)
    pub theme: Box<dyn Theme>,
    /// switch for enabling log splitting to `stderr`
    ///
    /// - `true`: log `trace` - `info` levels to `stdout` and `warn` - `error` levels to `stderr`
    ///
    /// - `false`: log all levels to `stdout`
    pub use_stderr: bool,
}

impl Default for Config {
    /// Return a `Config` with default values
    fn default() -> Config {
        Config {
            level: LevelFilter::Info,
            record_format: RecordFormat::Simple,
            color_format: Some(ColorFormat::Solid),
            theme: Box::new(Spectral {}),
            use_stderr: true,
        }
    }
}
