#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://i.imgur.com/xOjqfvy.gif",
    html_favicon_url = "https://i.imgur.com/Q7UUHoN.png",
    html_playground_url = "https://play.rust-lang.org/"
)]
#![deny(missing_docs)]

use colored::{Color, Colorize};
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use serde_json::json;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::sync::Mutex;
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;
use unicode_segmentation::UnicodeSegmentation;

pub mod color;
pub mod theme;
use theme::Theme;

/// Record formatting mode that determines how log records are structured
pub enum RecordFormat {
    /// JSON format
    Json,
    /// simple log format `<timestamp> [<target>] - <message>`
    Simple,
    /// custom record formatter provided by client code
    Custom(Box<dyn Sync + Send + Fn(&Record) -> String>),
}

/// Color formatting mode that determines how log records are colored/how a theme
/// is applied to log records
pub enum ColorFormat {
    /// solid color(s) applied from a theme to log lines
    Solid,
    /// linear color gradient applied over characters in a single line, with arg for number of steps
    /// in gradient (how many characters it will take to go from the starting color to the ending color
    /// for each level)
    InlineGradient(usize),
    /// linear color gradient applied over multiple lines, with arg for number of steps in gradient (how
    /// many lines it will take to go from the starting color to the ending color for each level)
    MultiLineGradient(usize),
}

/// Main configuration for a `DiscoLogger`
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
            theme: Box::new(theme::Simple {}),
            use_stderr: true,
        }
    }
}

/// RGB triplet
#[derive(Debug, PartialEq)]
pub struct Rgb {
    /// red intensity
    pub r: u8,
    /// green intensity
    pub g: u8,
    /// blue intensity
    pub b: u8,
}

impl Into<Color> for Rgb {
    /// Convert Rgb -> Color for easier use with string coloring
    fn into(self) -> Color {
        Color::TrueColor {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

/// RgbRange defines a linear color range from some start Rgb
/// triplet -> some end Rgb triplet
pub struct RgbRange {
    /// start of color range
    pub start: Rgb,
    /// end of color range
    pub end: Rgb,
}

/// Compute a new color `dist` distance along the linear
/// gradient from start to end of `range`
///
/// `dist` will be clamped to the range 0.0 - 1.0
///
/// # Arguments
///
/// * `range` - bounding color range for this linear gradient
/// * `dist` - desired distance along linear gradient (0.0 - 1.0)
fn linear_gradient(range: &RgbRange, dist: f32) -> Rgb {
    let dist = dist.clamp(0.0, 1.0);
    let start = &range.start;
    let end = &range.end;

    let r_range = (end.r as f32) - (start.r as f32);
    let g_range = (end.g as f32) - (start.g as f32);
    let b_range = (end.b as f32) - (start.b as f32);

    Rgb {
        r: ((start.r as f32) + (dist * r_range)) as u8,
        g: ((start.g as f32) + (dist * g_range)) as u8,
        b: ((start.b as f32) + (dist * b_range)) as u8,
    }
}

/// Get the distance, [0-1], that `x` falls along the line from 0-`n`
///
/// Dist will move in the direction:
/// - 0 -> 1 when (`x` % `2n`) <= `n`
/// - 1 -> 0 when `n` < (`x` % `2n`) < `2n`
///
/// If used with linear gradients, this oscillating dist will avoid the harsh visual
/// transition when wrapping around from 1 -> 0 (e.g. 0.9, 1.0, 0.0, 0.1 will not be
/// a smooth transition)
///
/// # Arguments
///
/// * `x` - some number whose value, `x` % `2n`, will be considered the distance along the line 0-`n`
/// * `n` - upper limit of range 0-`n`
fn oscillate_dist(x: usize, n: usize) -> f32 {
    let n = if n == 0 { 1 } else { n };
    (x.wrapping_add(n) % n.wrapping_mul(2)).abs_diff(n) as f32 / (n as f32)
}

/// Implements log::Log
pub struct DiscoLogger {
    /// Configuration for this logger
    config: Config,
    /// Count of how many lines are logged at each level,
    /// for use with coloring
    lines_logged: Mutex<HashMap<Level, usize>>,
    /// guard against interleaving from simultaneous writes to stdout + stderr
    write_mtx: Mutex<()>,
    /// handle to stdout
    stdout: io::Stdout,
    /// handle to stderr
    stderr: io::Stderr,
}

impl DiscoLogger {
    /// Create a new DiscoLogger with a default configuration
    pub fn new() -> DiscoLogger {
        DiscoLogger::with_config(Config::default())
    }

    /// Create a new DiscoLogger with a custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - configuration for this logger
    pub fn with_config(config: Config) -> DiscoLogger {
        DiscoLogger {
            config,
            lines_logged: Mutex::new(HashMap::new()),
            write_mtx: Mutex::new(()),
            stdout: io::stdout(),
            stderr: io::stderr(),
        }
    }

    /// Initialize this logger
    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_boxed_logger(Box::new(self)).map(|()| log::set_max_level(LevelFilter::Trace))
    }

    /// Convert a log record into a formatted string, based on the current logger configuration
    ///
    /// # Arguments
    ///
    /// * `record` - the log record to format
    fn format_record(&self, record: &Record) -> String {
        let now = OffsetDateTime::now_utc()
            .format(&Iso8601::DEFAULT)
            .expect("Failed to format time as ISO 8601");

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
    /// * `msg` - message to color
    /// * `level` - level of this log line
    fn color_solid(&self, msg: String, level: Level) -> String {
        let color = self.config.theme.solid(level);
        msg.color(color).to_string()
    }

    /// Apply linear color gradient across the graphemes in a string
    ///
    /// # Arguments
    ///
    /// * `msg` - message to color
    /// * `level` - level of this log line
    /// * `steps` - number of steps in gradient
    fn color_inline_gradient(&self, msg: String, level: Level, steps: usize) -> String {
        msg.graphemes(true)
            .enumerate()
            .map(|(i, c)| {
                let dist = oscillate_dist(i, steps);
                let color = linear_gradient(&self.config.theme.range(level), dist);
                c.color(color).to_string()
            })
            .collect::<Vec<String>>()
            .join("")
    }

    /// Apply a linear color gradient over multiple lines
    ///
    /// An independent linear color gradient will be applied across
    /// all lines logged at each level (e.g. `INFO` line color can change
    /// from green -> cyan as lines are logged, while lines logged at other
    /// levels move independently in their own gradient color ranges)
    ///
    /// # Arguments
    ///
    /// * `msg` - message to color
    /// * `level` - level of this log line
    /// * `steps` - number of steps in gradient
    fn color_multi_line_gradient(&self, msg: String, level: Level, steps: usize) -> String {
        let lines_logged = *self.lines_logged.lock().unwrap().entry(level).or_insert(0);
        let dist = oscillate_dist(lines_logged, steps);
        let color = linear_gradient(&self.config.theme.range(level), dist);
        msg.color(color).to_string()
    }

    /// Color a log line, based on the current logger configuration
    ///
    /// Arguments
    ///
    /// * `msg` - message to color
    /// * `level` - level of this log line
    fn color_log(&self, msg: String, level: Level) -> String {
        if self.config.color_format.is_none() {
            return msg;
        }

        let line = match self.config.color_format.as_ref().unwrap() {
            ColorFormat::Solid => self.color_solid(msg, level),
            ColorFormat::InlineGradient(steps) => self.color_inline_gradient(msg, level, *steps),
            ColorFormat::MultiLineGradient(steps) => {
                let l = self.color_multi_line_gradient(msg, level, *steps);

                // increment line counter for this level
                self.lines_logged
                    .lock()
                    .unwrap()
                    .entry(level)
                    .and_modify(|e| *e = e.wrapping_add(1))
                    .or_insert(0);

                return l;
            }
        };

        return line;
    }
}

impl Log for DiscoLogger {
    /// Check if this message should be logged
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.config.level
    }

    /// Log a message/record
    ///
    /// # Arguments
    ///
    /// * `record` - the record to log
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut msg = self.format_record(record);
            msg = self.color_log(msg, record.level());

            // stdout and stderr already have their own locks, but
            // there is nothing preventing logs simultaneously written
            // to stdout + stderr from being interleaved in the console
            //
            // this guard synchronizes writes so that stdout will not be
            // interleaved with stderr
            let _lk = self.write_mtx.lock().unwrap();

            match record.level() {
                Level::Warn | Level::Error => {
                    if self.config.use_stderr {
                        let _ = writeln!(self.stderr.lock(), "{}", msg.bold());
                    } else {
                        let _ = writeln!(self.stdout.lock(), "{}", msg.bold());
                    }
                }
                _ => {
                    let _ = writeln!(self.stdout.lock(), "{}", msg);
                }
            }
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use num::NumCast;

    // helpers

    /// Assert that two values are equal within some range, `eps`
    ///
    /// # Arguments
    ///
    /// * `a` - first value to compare
    /// * `b` - second value to compare
    /// * `eps` - max distance to consider diff of values equal within
    fn assert_eq_with_eps<T: NumCast>(a: T, b: T, eps: T) {
        let a: f64 = NumCast::from(a).unwrap();
        let b: f64 = NumCast::from(b).unwrap();
        let eps: f64 = NumCast::from(eps).unwrap();

        if (a - b).abs() > eps {
            panic!("{} and {} were not equal", a, b);
        }
    }

    /// To account for differences in the floating point math used to
    /// calculate colors along a gradient, this function compares the
    /// values in two Rgb structs within some range (+/- some value)
    ///
    /// # Arguments
    ///
    /// * `lhs` - first color in comparison
    /// * `rhs` - second color in comparison
    /// * `eps` - max distance to consider diff of r/g/b values equal within
    fn assert_rgb_eq(lhs: Rgb, rhs: Rgb, eps: Option<u8>) {
        let eps = eps.unwrap_or(1);

        assert_eq_with_eps(lhs.r, rhs.r, eps);
        assert_eq_with_eps(lhs.g, rhs.g, eps);
        assert_eq_with_eps(lhs.b, rhs.b, eps);
    }

    /// Assert that `f` gives a uniquely colored output for a string
    /// logged at each of the possible log levels
    ///
    /// # Arguments
    ///
    /// * `f` - function to color a string by log level
    fn assert_logs_colored_by_level(f: &dyn Fn(&DiscoLogger, String, Level) -> String) {
        // create a logger that will always log the same message "foo"
        // so that the only variable is changing color when comparing logs
        // at different levels
        let config = Config {
            level: LevelFilter::Trace,
            theme: Box::new(theme::Simple {}),
            record_format: RecordFormat::Custom(Box::new(|_| "foo".to_string())),
            ..Default::default()
        };
        let logger = DiscoLogger::with_config(config);

        // run `f` on `msg` with each level to make sure that
        // no two levels give the same colored output
        let msg = "foo".to_string();
        let lines = [
            f(&logger, msg.clone(), Level::Trace),
            f(&logger, msg.clone(), Level::Debug),
            f(&logger, msg.clone(), Level::Info),
            f(&logger, msg.clone(), Level::Warn),
            f(&logger, msg.clone(), Level::Error),
        ];

        // check that each colored line is unique
        for (i, line) in lines.iter().enumerate() {
            for line1 in lines.iter().skip(i + 1) {
                if line == line1 {
                    panic!("\"{}\" and \"{}\" had different levels but generated the same formatted line", line, line1);
                }
            }
        }
    }

    // tests

    #[test]
    fn rgb_into_color_is_accurate() {
        let test_cases = vec![
            (
                Rgb { r: 0, g: 0, b: 0 },
                Color::TrueColor { r: 0, g: 0, b: 0 },
            ),
            (
                Rgb {
                    r: 127,
                    g: 128,
                    b: 129,
                },
                Color::TrueColor {
                    r: 127,
                    g: 128,
                    b: 129,
                },
            ),
            (
                Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                Color::TrueColor {
                    r: 255,
                    g: 255,
                    b: 255,
                },
            ),
        ];

        for (rgb, tc) in test_cases {
            let c: Color = rgb.into();
            assert_eq!(c, tc);
        }
    }

    #[test]
    fn linear_gradient_calculates_correct_color() {
        let r = RgbRange {
            start: Rgb { r: 0, g: 0, b: 0 },
            end: Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
        };

        assert_rgb_eq(linear_gradient(&r, 0.0), Rgb { r: 0, g: 0, b: 0 }, None);
        assert_rgb_eq(
            linear_gradient(&r, 0.25),
            Rgb {
                r: 64,
                g: 64,
                b: 64,
            },
            None,
        );
        assert_rgb_eq(
            linear_gradient(&r, 0.5),
            Rgb {
                r: 128,
                g: 128,
                b: 128,
            },
            None,
        );
        assert_rgb_eq(
            linear_gradient(&r, 0.75),
            Rgb {
                r: 190,
                g: 190,
                b: 190,
            },
            None,
        );
        assert_rgb_eq(
            linear_gradient(&r, 1.0),
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
        let r = RgbRange {
            start: Rgb { r: 0, g: 0, b: 0 },
            end: Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
        };

        let expected = Rgb { r: 0, g: 0, b: 0 };
        assert_rgb_eq(linear_gradient(&r, -1.0), expected, None);

        let expected = Rgb {
            r: 255,
            g: 255,
            b: 255,
        };
        assert_rgb_eq(linear_gradient(&r, 100.0), expected, None);
    }

    #[test]
    fn oscillate_dist_oscillates() {
        assert_eq_with_eps(oscillate_dist(0, 255), 0.0, 1e-2);
        assert_eq_with_eps(oscillate_dist(128, 255), 0.5, 1e-2);
        assert_eq_with_eps(oscillate_dist(255, 255), 1.0, 1e-2);
        assert_eq_with_eps(oscillate_dist(256, 255), 1.0 - (1.0 / 255.0), 1e-2);
        assert_eq_with_eps(oscillate_dist(383, 255), 0.5, 1e-2);
        assert_eq_with_eps(oscillate_dist(510, 255), 0.0, 1e-2);
        assert_eq_with_eps(oscillate_dist(638, 255), 0.5, 1e-2);
        assert_eq_with_eps(oscillate_dist(765, 255), 1.0, 1e-2);
        assert_eq_with_eps(
            oscillate_dist(usize::max_value(), 255),
            oscillate_dist(usize::max_value() - 255, 255),
            1e-2,
        );
        assert_eq_with_eps(oscillate_dist(12, usize::max_value()), 1.0, 1e-2);
        assert_eq_with_eps(oscillate_dist(257, usize::max_value()), 1.0, 1e-2);
        assert_eq_with_eps(
            oscillate_dist(usize::max_value(), usize::max_value()),
            1.0,
            1e-2,
        );
    }

    #[test]
    fn oscillate_dist_handles_0_n() {
        // this shouldn't panic
        oscillate_dist(0, 0);
    }

    #[test]
    fn format_record_presets_return_non_empty() {
        for fmt in vec![RecordFormat::Json, RecordFormat::Simple] {
            let config = Config {
                record_format: fmt,
                ..Default::default()
            };
            let logger = DiscoLogger::with_config(config);

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
            let config = Config {
                record_format: fmt,
                ..Default::default()
            };
            let logger = DiscoLogger::with_config(config);

            assert_eq!(logger.format_record(&rec), expected);
        }
    }

    #[test]
    fn color_solid_colors_by_level() {
        assert_logs_colored_by_level(&DiscoLogger::color_solid);
    }

    #[test]
    fn color_inline_gradient_colors_by_level() {
        let color_fn = |logger: &DiscoLogger, msg: String, level: Level| -> String {
            logger.color_inline_gradient(msg, level, 20)
        };

        assert_logs_colored_by_level(&color_fn);
    }

    #[test]
    fn color_multi_line_gradient_colors_by_level() {
        let color_fn = |logger: &DiscoLogger, msg: String, level: Level| -> String {
            logger.color_multi_line_gradient(msg, level, 20)
        };

        assert_logs_colored_by_level(&color_fn);
    }

    #[test]
    fn color_fns_handle_empty_msg() {
        let config = Config {
            level: LevelFilter::Trace,
            ..Default::default()
        };
        let logger = DiscoLogger::with_config(config);

        // none of these calls should panic with an empty message
        logger.color_solid("".to_string(), Level::Warn);
        logger.color_inline_gradient("".to_string(), Level::Warn, 10);
        logger.color_multi_line_gradient("".to_string(), Level::Warn, 10);
    }

    #[test]
    fn color_log_with_none_format_returns_orig() {
        let config = Config {
            color_format: None,
            ..Default::default()
        };
        let logger = DiscoLogger::with_config(config);

        // input msg should not be altered by None color format
        let msg = "foo".to_string();
        assert_eq!(logger.color_log(msg.clone(), Level::Info), msg);
    }

    #[test]
    fn color_log_with_inline_gradient_uses_steps_arg() {
        let config = Config {
            color_format: Some(ColorFormat::InlineGradient(2)),
            theme: Box::new(theme::Simple {}),
            ..Default::default()
        };
        let logger = DiscoLogger::with_config(config);

        let msgs = vec!["0000000000".to_string(), "नमस्तेनमस्तेनमस्तेनमस्तेनमस्ते".to_string()];

        for msg in msgs {
            let msg_colored = logger.color_log(msg.clone(), Level::Info);

            // collect ANSI 24-bit escape sequences to compare color of each grapheme
            let words = msg_colored.unicode_words().collect::<Vec<&str>>();
            let num_words = words.len();
            let graphemes = msg.graphemes(true).collect::<Vec<&str>>().len();
            let escape_seqs = words
                .into_iter()
                .step_by(num_words / graphemes)
                .collect::<Vec<&str>>();

            // check that gradient restarts at index 4 (2 * steps)
            //
            // it restarts at 4 instead of 2 because the gradient is first
            // traversed from start to end, then from end to start, then start to end
            // again, oscillating this way indefinitely
            assert_ne!(escape_seqs[0], escape_seqs[1]);
            assert_eq!(escape_seqs[0], escape_seqs[4]);
            assert_eq!(escape_seqs[1], escape_seqs[5]);
            assert_eq!(escape_seqs[2], escape_seqs[6]);
            assert_eq!(escape_seqs[3], escape_seqs[7]);
        }
    }

    #[test]
    fn color_log_with_multi_line_gradient_changes_color_within_level() {
        let config = Config {
            color_format: Some(ColorFormat::MultiLineGradient(20)),
            theme: Box::new(theme::Simple {}),
            ..Default::default()
        };
        let logger = DiscoLogger::with_config(config);
        let msg = "foo".to_string();

        // the color should change each time a message is logged,
        // since the multi-line gradient color format should create
        // a color gradient over multiple lines (under each level)
        let assert_color_changes_within_level = |level: Level| {
            let mut last_logged = "".to_string();

            for _ in 0..10 {
                let l = logger.color_log(msg.clone(), level);
                assert_ne!(last_logged, l);
                last_logged = l;
            }
        };

        assert_color_changes_within_level(Level::Trace);
        assert_color_changes_within_level(Level::Debug);
        assert_color_changes_within_level(Level::Info);
        assert_color_changes_within_level(Level::Warn);
        assert_color_changes_within_level(Level::Error);
    }

    #[test]
    fn color_log_with_multi_line_gradient_uses_steps_arg() {
        // use multi-line gradient with 2 steps in the linear gradient
        let steps: usize = 2;

        let config = Config {
            color_format: Some(ColorFormat::MultiLineGradient(steps)),
            theme: Box::new(theme::Simple {}),
            record_format: RecordFormat::Custom(Box::new(|_| "foo".to_string())),
            ..Default::default()
        };
        let logger = DiscoLogger::with_config(config);
        let msg = "foo".to_string();

        let lines = vec![
            // gradient starts going from start -> end here
            logger.color_log(msg.clone(), Level::Info),
            logger.color_log(msg.clone(), Level::Info),
            // end -> start
            logger.color_log(msg.clone(), Level::Info),
            logger.color_log(msg.clone(), Level::Info),
            // gradient should start over here, start -> end
            logger.color_log(msg.clone(), Level::Info),
            logger.color_log(msg.clone(), Level::Info),
            // end -> start
            logger.color_log(msg.clone(), Level::Info),
            logger.color_log(msg.clone(), Level::Info),
        ];

        // check that gradient restarts at index 4 (2 * steps)
        //
        // it restarts at 4 instead of 2 because the gradient is first
        // traversed from start to end, then from end to start, then start to end
        // again, oscillating this way indefinitely
        assert_ne!(lines[0], lines[1]);
        assert_eq!(lines[0], lines[4]);
        assert_eq!(lines[1], lines[5]);
        assert_eq!(lines[2], lines[6]);
        assert_eq!(lines[3], lines[7]);
    }

    #[test]
    fn enabled_filters_levels() {
        let config = Config {
            level: LevelFilter::Warn,
            ..Default::default()
        };
        let logger = DiscoLogger::with_config(config);
        let mut mb = Metadata::builder();

        assert!(!logger.enabled(&mut mb.level(Level::Trace).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Debug).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Info).build()));
        assert!(logger.enabled(&mut mb.level(Level::Warn).build()));
        assert!(logger.enabled(&mut mb.level(Level::Error).build()));
    }

    #[test]
    fn enabled_with_off_level_filter_is_always_false() {
        let config = Config {
            level: LevelFilter::Off,
            ..Default::default()
        };
        let logger = DiscoLogger::with_config(config);
        let mut mb = Metadata::builder();

        assert!(!logger.enabled(&mut mb.level(Level::Trace).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Debug).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Info).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Warn).build()));
        assert!(!logger.enabled(&mut mb.level(Level::Error).build()));
    }

    #[test]
    fn log_handles_empty_record() {
        let config = Config::default();
        let logger = DiscoLogger::with_config(config);

        // create record with fields set to empty strings
        let rec = Record::builder()
            .args(format_args!(""))
            .level(Level::Info)
            .target("")
            .build();

        // this shouldn't panic
        logger.log(&rec);
    }
}
