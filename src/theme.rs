use crate::color::Color;
use crate::{Rgb, RgbRange};
use log::Level;

/// Define a color range for each log level
/// to use as a theme with this library's color
/// formats
pub trait Theme: Send + Sync {
    /// return the representative solid color for this theme at each level
    fn solid(&self, level: Level) -> Rgb;
    /// return the bounding color range for this theme at each level
    fn range(&self, level: Level) -> RgbRange;
}

pub struct NormalTheme {}

impl Theme for NormalTheme {
    fn solid(&self, level: Level) -> Rgb {
        self.range(level).start
    }

    fn range(&self, level: Level) -> RgbRange {
        match level {
            Level::Trace => RgbRange {
                start: Color::DarkPink.value(),
                end: Color::Pink.value(),
            },
            Level::Debug => RgbRange {
                start: Color::DarkCyan.value(),
                end: Color::Cyan.value(),
            },
            Level::Info => RgbRange {
                start: Color::DarkGreen.value(),
                end: Color::Green.value(),
            },
            Level::Warn => RgbRange {
                start: Color::DarkOrange.value(),
                end: Color::Orange.value(),
            },
            Level::Error => RgbRange {
                start: Color::DarkRed.value(),
                end: Color::Red.value(),
            },
        }
    }
}
