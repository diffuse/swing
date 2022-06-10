use chrono::prelude::*;
use colored::Colorize;
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};

mod colors;
use colors::Color;

/// Record formatting mode
pub enum RecordFormat {
    Json,
    Simple,
    Custom(Box<dyn Sync + Send + Fn(&Record) -> String>),
}

/// Color formatting mode
pub enum ColorFormat {
    Solid,
    InlineGradient,
    MultiLineGradient,
}

/// Config for logger
pub struct LoggerConfig {
    /// log level
    pub level: LevelFilter,
    /// record formatting mode
    pub record_format: RecordFormat,
    /// color formatting mode
    pub color_format: Option<ColorFormat>,
    /// color theme
    pub theme: Box<dyn Theme>,
}

impl Default for LoggerConfig {
    fn default() -> LoggerConfig {
        LoggerConfig {
            level: LevelFilter::Info,
            record_format: RecordFormat::Json,
            color_format: Some(ColorFormat::Solid),
            theme: Box::new(NormalTheme {}),
        }
    }
}

/// RGB triplet
#[derive(Debug, PartialEq)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

/// Define a color range to use as a theme
/// with this library's color formats
pub trait Theme: Send + Sync {
    // TODO rename these methods to normal/start/end
    //
    // TODO rename this/reconsider this design
    fn normal_color(&self, level: Level) -> Rgb;
    /// return the starting color in this theme's color range
    fn start_color(&self, level: Level) -> Rgb;
    /// return the ending color in this theme's color range
    fn end_color(&self, level: Level) -> Rgb;
}

pub struct NormalTheme {}

impl Theme for NormalTheme {
    fn normal_color(&self, level: Level) -> Rgb {
        self.start_color(level)
    }

    fn start_color(&self, level: Level) -> Rgb {
        match level {
            Level::Trace => Color::DarkPink.value(),
            Level::Debug => Color::DarkCyan.value(),
            Level::Info => Color::DarkGreen.value(),
            Level::Warn => Color::DarkOrange.value(),
            Level::Error => Color::DarkRed.value(),
        }
    }

    fn end_color(&self, level: Level) -> Rgb {
        match level {
            Level::Trace => Color::Pink.value(),
            Level::Debug => Color::Cyan.value(),
            Level::Info => Color::Green.value(),
            Level::Warn => Color::Orange.value(),
            Level::Error => Color::Red.value(),
        }
    }
}

/// Compute a new color `dist` distance along the linear
/// gradient from `start` to `end`
///
/// `dist` will be clamped to the range 0.0 - 1.0
///
/// # Arguments
///
/// * `start` - starting color for linear gradient
/// * `end` - ending color for linear gradient
/// * `dist` - desired distance along linear gradient (0.0 - 1.0)
fn linear_gradient(start: &Rgb, end: &Rgb, dist: f32) -> Rgb {
    let dist = dist.clamp(0.0, 1.0);

    let r_range = (end.r as f32) - (start.r as f32);
    let g_range = (end.g as f32) - (start.g as f32);
    let b_range = (end.b as f32) - (start.b as f32);

    Rgb {
        r: ((start.r as f32) + (dist * r_range)) as u8,
        g: ((start.g as f32) + (dist * g_range)) as u8,
        b: ((start.b as f32) + (dist * b_range)) as u8,
    }
}

/// Implements log::Log
pub struct DiscoLogger {
    config: LoggerConfig,
    lines_logged: AtomicUsize,
}

impl DiscoLogger {
    /// Create a new DiscoLogger
    ///
    /// # Arguments
    ///
    /// * `config` - configuration for this logger
    pub fn new(config: LoggerConfig) -> DiscoLogger {
        DiscoLogger {
            config,
            lines_logged: AtomicUsize::new(0),
        }
    }

    /// Initialize this logger
    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_boxed_logger(Box::new(self)).map(|()| log::set_max_level(LevelFilter::Trace))
    }

    /// Format a log message based on the current RecordFormat setting
    fn format_record(&self, record: &Record) -> String {
        let now = Utc::now().to_rfc3339();

        match &self.config.record_format {
            RecordFormat::Json => json!({
                "time": now,
                "level": record.level(),
                "target": record.target(),
                "message": record.args(),
            })
            .to_string(),
            RecordFormat::Simple => {
                format!(
                    "{} [{}] {} - {}",
                    now,
                    record.target(),
                    record.level(),
                    record.args()
                )
            }
            RecordFormat::Custom(f) => f(record),
        }
    }

    /// Color code strings using one color per line,
    /// chosen based on log level
    ///
    /// # Arguments
    ///
    /// * `level` - level of this log line
    /// * `msg` - messsage being logged
    fn color_solid(&self, level: Level, msg: String) -> String {
        let color = self.config.theme.normal_color(level);

        let true_color = colored::Color::TrueColor {
            r: color.r,
            g: color.g,
            b: color.b,
        };

        msg.color(true_color).to_string()
    }

    /// Apply linear color gradient over each line
    ///
    /// # Arguments
    ///
    /// * `level` - level of this log line
    /// * `msg` - messsage being logged
    fn color_inline_gradient(&self, level: Level, msg: String) -> String {
        let theme = &self.config.theme;

        msg.chars()
            .enumerate()
            .map(|(i, c)| {
                // how far along the linear gradient this color should be (0.0 - 1.0)
                let dist = (i as f32) / (msg.len() as f32);
                let color =
                    linear_gradient(&theme.start_color(level), &theme.end_color(level), dist);

                let true_color = colored::Color::TrueColor {
                    r: color.r,
                    g: color.g,
                    b: color.b,
                };

                c.to_string().color(true_color).to_string()
            })
            .collect::<Vec<String>>()
            .join("")
    }

    /// Apply a linear color gradient over multiple lines
    ///
    /// # Arguments
    ///
    /// * `level` - level of this log line
    /// * `msg` - messsage being logged
    fn color_multi_line_gradient(&self, level: Level, msg: String) -> String {
        // essentially a linear gradient, but lines move along the gradient in ((i % N) / N) jumps
        let n = 20;
        let dist = (self.lines_logged.load(Ordering::SeqCst) % n) as f32 / n as f32;
        let theme = &self.config.theme;
        let color = linear_gradient(&theme.start_color(level), &theme.end_color(level), dist);

        let true_color = colored::Color::TrueColor {
            r: color.r,
            g: color.g,
            b: color.b,
        };

        msg.color(true_color).to_string()
    }

    /// Color a log line based on selected options
    ///
    /// Arguments
    ///
    /// * `msg` - message being logged
    /// * `record` - log record
    fn color_log(&self, msg: String, record: &Record) -> String {
        if self.config.color_format.is_none() {
            return msg;
        }

        let s = match self.config.color_format.as_ref().unwrap() {
            ColorFormat::Solid => self.color_solid(record.level(), msg),
            ColorFormat::InlineGradient => self.color_inline_gradient(record.level(), msg),
            ColorFormat::MultiLineGradient => self.color_multi_line_gradient(record.level(), msg),
        };

        self.lines_logged.fetch_add(1, Ordering::SeqCst);
        return s;
    }
}

impl Log for DiscoLogger {
    /// Check if this message should be logged
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.config.level
    }

    /// Log a message
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut msg = self.format_record(record);
            msg = self.color_log(msg, record);

            match record.level() {
                Level::Warn | Level::Error => eprintln!("{}", msg.bold()),
                _ => println!("{}", msg),
            }
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabled_filters_levels() {
        // TODO test LevelFilter::Off

        let config = LoggerConfig {
            level: LevelFilter::Warn,
            ..Default::default()
        };
        let logger = DiscoLogger::new(config);
        let mut mb = Metadata::builder();

        assert!(!logger.enabled(&mut mb.level(Level::Trace).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Debug).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Info).build()));
        assert!(logger.enabled(&mut mb.level(Level::Warn).build()));
        assert!(logger.enabled(&mut mb.level(Level::Error).build()));
    }

    #[test]
    fn format_record_presets_return_non_empty() {
        for fmt in vec![RecordFormat::Json, RecordFormat::Simple] {
            let config = LoggerConfig {
                record_format: fmt,
                ..Default::default()
            };
            let logger = DiscoLogger::new(config);

            // create normal test record
            let rec = Record::builder()
                .args(format_args!("foo"))
                .level(Level::Info)
                .target("test")
                .build();

            assert!(!logger.format_record(&rec).is_empty());

            // create record with empty args and target
            let rec = Record::builder()
                .args(format_args!(""))
                .level(Level::Info)
                .target("")
                .build();

            // record should still give non-empty log lines
            assert!(!logger.format_record(&rec).is_empty());
        }
    }

    #[test]
    fn format_record_custom_formats_correctly() {
        let test_cases = vec![
            (RecordFormat::Custom(Box::new(|_| "".to_string())), ""),
            (
                RecordFormat::Custom(Box::new(|r| format!("{} {}", r.level(), r.args()))),
                "INFO foo",
            ),
            (
                RecordFormat::Custom(Box::new(|r| {
                    format!("{} [{}] {}", r.level(), r.target(), r.args())
                })),
                "INFO [test] foo",
            ),
        ];

        let rec = Record::builder()
            .args(format_args!("foo"))
            .level(Level::Info)
            .target("test")
            .build();

        for (fmt, expected) in test_cases {
            let config = LoggerConfig {
                record_format: fmt,
                ..Default::default()
            };
            let logger = DiscoLogger::new(config);

            assert_eq!(logger.format_record(&rec), expected);
        }
    }

    /// To account for differences in the floating point math used to
    /// calculate colors along a gradient, this function compares the
    /// values in two Rgb structs within some range (+/- some value)
    ///
    /// # Arguments
    ///
    /// * `valid_range` - Rgb values +/- this value will be considered equal
    fn assert_rgb_eq(lhs: Rgb, rhs: Rgb, valid_range: Option<u8>) {
        let valid_range = valid_range.unwrap_or(1);

        let is_eq = |a: u8, b: u8| -> bool {
            // cast to avoid overflow
            let a = a as i32;
            let b = b as i32;
            let valid_range = valid_range as i32;

            // true if a within b +/- valid_range
            !(a < (b - valid_range) || a > (b + valid_range))
        };

        assert!(is_eq(lhs.r, rhs.r) && is_eq(lhs.g, rhs.g) && is_eq(lhs.b, rhs.b))
    }

    #[test]
    fn linear_gradient_calculates_correct_color() {
        let start = Rgb { r: 0, g: 0, b: 0 };
        let end = Rgb {
            r: 255,
            g: 255,
            b: 255,
        };

        assert_rgb_eq(
            linear_gradient(&start, &end, 0.0),
            Rgb { r: 0, g: 0, b: 0 },
            None,
        );
        assert_rgb_eq(
            linear_gradient(&start, &end, 0.25),
            Rgb {
                r: 64,
                g: 64,
                b: 64,
            },
            None,
        );
        assert_rgb_eq(
            linear_gradient(&start, &end, 0.5),
            Rgb {
                r: 128,
                g: 128,
                b: 128,
            },
            None,
        );
        assert_rgb_eq(
            linear_gradient(&start, &end, 0.75),
            Rgb {
                r: 190,
                g: 190,
                b: 190,
            },
            None,
        );
        assert_rgb_eq(
            linear_gradient(&start, &end, 1.0),
            Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
            None,
        );
    }

    #[test]
    fn linear_gradient_clamps_dist() {
        let start = Rgb { r: 0, g: 0, b: 0 };
        let end = Rgb {
            r: 255,
            g: 255,
            b: 255,
        };

        let expected = Rgb { r: 0, g: 0, b: 0 };
        assert_rgb_eq(linear_gradient(&start, &end, -1.0), expected, None);

        let expected = Rgb {
            r: 255,
            g: 255,
            b: 255,
        };
        assert_rgb_eq(linear_gradient(&start, &end, 100.0), expected, None);
    }

    #[test]
    fn color_solid_colors_by_level() {
        let config = LoggerConfig {
            level: LevelFilter::Trace,
            ..Default::default()
        };
        let logger = DiscoLogger::new(config);

        let msg = "foo".to_string();
        let lines = [
            logger.color_solid(Level::Trace, msg.clone()),
            logger.color_solid(Level::Debug, msg.clone()),
            logger.color_solid(Level::Info, msg.clone()),
            logger.color_solid(Level::Warn, msg.clone()),
            logger.color_solid(Level::Error, msg.clone()),
        ];

        for (i, line) in lines.iter().enumerate() {
            for line1 in lines.iter().skip(i + 1) {
                if line == line1 {
                    panic!("\"{}\" and \"{}\" had different levels but generated the same formatted line", line, line1);
                }
            }
        }
    }

    #[test]
    fn color_solid_handles_empty_msg() {
        let config = LoggerConfig {
            level: LevelFilter::Trace,
            ..Default::default()
        };
        let logger = DiscoLogger::new(config);

        logger.color_solid(Level::Warn, "".to_string());
    }

    #[test]
    fn color_inline_gradient_colors_by_level() {
        let config = LoggerConfig {
            level: LevelFilter::Trace,
            ..Default::default()
        };
        let logger = DiscoLogger::new(config);

        let msg = "foo".to_string();
        let lines = [
            logger.color_inline_gradient(Level::Trace, msg.clone()),
            logger.color_inline_gradient(Level::Debug, msg.clone()),
            logger.color_inline_gradient(Level::Info, msg.clone()),
            logger.color_inline_gradient(Level::Warn, msg.clone()),
            logger.color_inline_gradient(Level::Error, msg.clone()),
        ];

        for (i, line) in lines.iter().enumerate() {
            for line1 in lines.iter().skip(i + 1) {
                if line == line1 {
                    panic!("\"{}\" and \"{}\" had different levels but generated the same formatted line", line, line1);
                }
            }
        }
    }

    #[test]
    fn color_inline_gradient_handles_empty_msg() {
        let config = LoggerConfig {
            level: LevelFilter::Trace,
            ..Default::default()
        };
        let logger = DiscoLogger::new(config);

        logger.color_inline_gradient(Level::Warn, "".to_string());
    }

    #[test]
    fn color_log_with_none_format_returns_orig() {
        let config = LoggerConfig {
            color_format: None,
            ..Default::default()
        };
        let logger = DiscoLogger::new(config);

        let rec = Record::builder()
            .args(format_args!("foo"))
            .level(Level::Info)
            .target("test")
            .build();

        // input msg should not be altered by None color format
        let msg = "foo".to_string();
        assert_eq!(logger.color_log(msg.clone(), &rec), msg);
    }
}
