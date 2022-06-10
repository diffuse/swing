use crate::color::Color;
use crate::Rgb;
use log::Level;

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
