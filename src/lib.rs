use chrono::prelude::*;
use colored::{Color, Colorize};
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};

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
}

impl Default for LoggerConfig {
    fn default() -> LoggerConfig {
        LoggerConfig {
            level: LevelFilter::Info,
            record_format: RecordFormat::Json,
            color_format: Some(ColorFormat::Solid),
        }
    }
}

/// RGB values 0-255 in floating point format
#[derive(Debug, PartialEq)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

/// Get RGB values corresponding to some known color
///
/// Bright white will be returned if `color` is unknown
///
/// # Arguments
///
/// * `color` - color name to retreive RGB values for
fn rgb_from_str(color: &str) -> Rgb {
    match color {
        "magenta" => Rgb {
            r: 149,
            g: 119,
            b: 149,
        },
        "bright magenta" => Rgb {
            r: 227,
            g: 184,
            b: 227,
        },
        "cyan" => Rgb {
            r: 13,
            g: 144,
            b: 138,
        },
        "bright cyan" => Rgb {
            r: 20,
            g: 219,
            b: 210,
        },
        "green" => Rgb {
            r: 70,
            g: 140,
            b: 10,
        },
        "bright green" => Rgb {
            r: 107,
            g: 217,
            b: 13,
        },
        "orange" => Rgb {
            r: 255,
            g: 128,
            b: 0,
        },
        "bright orange" => Rgb {
            r: 247,
            g: 178,
            b: 109,
        },
        "red" => Rgb {
            r: 203,
            g: 0,
            b: 11,
        },
        "bright red" => Rgb {
            r: 252,
            g: 58,
            b: 69,
        },
        _ => Rgb {
            r: 255,
            g: 255,
            b: 255,
        },
    }
}

/// Color code strings using one color per line,
/// chosen based on log level
///
/// # Arguments
///
/// * `level` - level of this log line
/// * `msg` - messsage being logged
fn color_solid(level: Level, msg: String) -> String {
    match level {
        Level::Trace => msg.bright_magenta(),
        Level::Debug => msg.cyan(),
        Level::Info => msg.green(),
        Level::Warn => msg.truecolor(255, 128, 0),
        Level::Error => msg.red(),
    }
    .to_string()
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

/// Apply linear color gradient over each line
///
/// # Arguments
///
/// * `level` - level of this log line
/// * `msg` - messsage being logged
fn color_inline_gradient(level: Level, msg: String) -> String {
    let (start_color, end_color) = match level {
        Level::Trace => (rgb_from_str("magenta"), rgb_from_str("bright magenta")),
        Level::Debug => (rgb_from_str("cyan"), rgb_from_str("bright cyan")),
        Level::Info => (rgb_from_str("green"), rgb_from_str("bright green")),
        Level::Warn => (rgb_from_str("orange"), rgb_from_str("bright orange")),
        _ => (rgb_from_str("red"), rgb_from_str("bright red")),
    };

    msg.chars()
        .enumerate()
        .map(|(i, c)| {
            // how far along the linear gradient this color should be (0.0 - 1.0)
            let dist = (i as f32) / (msg.len() as f32);
            let color = linear_gradient(&start_color, &end_color, dist);

            let true_color = Color::TrueColor {
                r: color.r,
                g: color.g,
                b: color.b,
            };

            c.to_string().color(true_color).to_string()
        })
        .collect::<Vec<String>>()
        .join("")
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

    /// Apply a linear color gradient over multiple lines
    ///
    /// # Arguments
    ///
    /// * `level` - level of this log line
    /// * `msg` - messsage being logged
    fn color_multi_line_gradient(&self, level: Level, msg: String) -> String {
        let (start_color, end_color) = match level {
            Level::Trace => (rgb_from_str("magenta"), rgb_from_str("bright magenta")),
            Level::Debug => (rgb_from_str("cyan"), rgb_from_str("bright cyan")),
            Level::Info => (rgb_from_str("bright green"), rgb_from_str("cyan")), //rgb_from_str("bright green")),
            Level::Warn => (rgb_from_str("orange"), rgb_from_str("bright orange")),
            _ => (rgb_from_str("red"), rgb_from_str("bright red")),
        };

        // essentially a linear gradient, but lines move along the gradient in ((i % N) / N) jumps
        let n = 20;
        let dist = (self.lines_logged.load(Ordering::SeqCst) % n) as f32 / n as f32;
        let color = linear_gradient(&start_color, &end_color, dist);

        let true_color = Color::TrueColor {
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
    /// * `color_format` - formatting to use for color
    fn color_log(
        &self,
        msg: String,
        record: &Record,
        color_format: &Option<ColorFormat>,
    ) -> String {
        if color_format.is_none() {
            return msg;
        }

        let s = match color_format.as_ref().unwrap() {
            ColorFormat::Solid => color_solid(record.level(), msg),
            ColorFormat::InlineGradient => color_inline_gradient(record.level(), msg),
            ColorFormat::MultiLineGradient => self.color_multi_line_gradient(record.level(), msg),
        };

        self.lines_logged.fetch_add(1, Ordering::SeqCst);
        return s;
    }
}

/// Format a log message based on the current RecordFormat setting
fn format_record(record: &Record, record_format: &RecordFormat) -> String {
    let now = Utc::now().to_rfc3339();

    match record_format {
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

impl Log for DiscoLogger {
    /// Check if this message should be logged
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.config.level
    }

    /// Log a message
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut msg = format_record(record, &self.config.record_format);
            msg = self.color_log(msg, record, &self.config.color_format);

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
        // create normal test record
        let rec = Record::builder()
            .args(format_args!("foo"))
            .level(Level::Info)
            .target("test")
            .build();

        // record should give non-empty log line
        assert!(!format_record(&rec, &RecordFormat::Json).is_empty());
        assert!(!format_record(&rec, &RecordFormat::Simple).is_empty());

        // create record with empty args and target
        let rec = Record::builder()
            .args(format_args!(""))
            .level(Level::Info)
            .target("")
            .build();

        // record should still give non-empty log lines
        assert!(!format_record(&rec, &RecordFormat::Json).is_empty());
        assert!(!format_record(&rec, &RecordFormat::Simple).is_empty());
    }

    #[test]
    fn format_record_custom_formats_correctly() {
        let rec = Record::builder()
            .args(format_args!("foo"))
            .level(Level::Info)
            .target("test")
            .build();

        assert_eq!(
            format_record(&rec, &RecordFormat::Custom(Box::new(|_| "".to_string()))),
            ""
        );

        assert_eq!(
            format_record(
                &rec,
                &RecordFormat::Custom(Box::new(|r| format!("{} {}", r.level(), r.args()))),
            ),
            "INFO foo"
        );

        assert_eq!(
            format_record(
                &rec,
                &RecordFormat::Custom(Box::new(|r| format!(
                    "{} [{}] {}",
                    r.level(),
                    r.target(),
                    r.args()
                ))),
            ),
            "INFO [test] foo"
        );
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
        let msg = "foo".to_string();
        let lines = [
            color_solid(Level::Trace, msg.clone()),
            color_solid(Level::Debug, msg.clone()),
            color_solid(Level::Info, msg.clone()),
            color_solid(Level::Warn, msg.clone()),
            color_solid(Level::Error, msg.clone()),
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
        color_solid(Level::Warn, "".to_string());
    }

    #[test]
    fn color_inline_gradient_colors_by_level() {
        let msg = "foo".to_string();
        let lines = [
            color_inline_gradient(Level::Trace, msg.clone()),
            color_inline_gradient(Level::Debug, msg.clone()),
            color_inline_gradient(Level::Info, msg.clone()),
            color_inline_gradient(Level::Warn, msg.clone()),
            color_inline_gradient(Level::Error, msg.clone()),
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
        color_inline_gradient(Level::Warn, "".to_string());
    }

    /*
    TODO
    #[test]
    fn color_log_with_none_format_returns_orig() {
        let rec = Record::builder()
            .args(format_args!("foo"))
            .level(Level::Info)
            .target("test")
            .build();

        let msg = "foo".to_string();

        assert_eq!(color_log(msg.clone(), &rec, &None), msg);
    }
    */

    #[test]
    fn rgb_from_str_defaults_to_white() {
        assert_eq!(
            rgb_from_str("foo"),
            Rgb {
                r: 255,
                g: 255,
                b: 255
            }
        );
    }
}
