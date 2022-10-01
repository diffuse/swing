use crate::color::Color;
use crate::{Rgb, RgbRange};
use log::Level;

/// Define a log level specific color palette to be injected into
/// color formatting
pub trait Theme: Send + Sync {
    /// return the representative solid color for this theme at each level
    fn solid(&self, level: Level) -> Rgb;
    /// return the bounding color range for this theme at each level
    fn range(&self, level: Level) -> RgbRange;
}

/// Basic log level colors
pub struct Simple {}

impl Theme for Simple {
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

/// Move down the color spectrum by level
pub struct Spectral {}

impl Theme for Spectral {
    fn solid(&self, level: Level) -> Rgb {
        self.range(level).end
    }

    fn range(&self, level: Level) -> RgbRange {
        match level {
            Level::Trace => RgbRange {
                start: Color::DarkMagenta.value(),
                end: Color::Pink.value(),
            },
            Level::Debug => RgbRange {
                start: Color::DarkBlue.value(),
                end: Color::Cyan.value(),
            },
            Level::Info => RgbRange {
                start: Color::DarkCyan.value(),
                end: Color::Green.value(),
            },
            Level::Warn => RgbRange {
                start: Color::DarkYellow.value(),
                end: Color::Orange.value(),
            },
            Level::Error => RgbRange {
                start: Color::DarkOrange.value(),
                end: Color::Red.value(),
            },
        }
    }
}
