use disco::{Color, ColorFormat, Config, DiscoLogger, Rgb, RgbRange, Theme};
use log::{Level, LevelFilter};

/// Custom theme
pub struct MyTheme {}

impl Theme for MyTheme {
    /// This method returns the color that will be used for each
    /// level when the Solid color format is chosen
    ///
    /// This implementation simply uses the start color
    /// from each level's color range as its solid color, but you
    /// can perform another level match here and choose whichever
    /// specific colors you would like.
    fn solid(&self, level: Level) -> Rgb {
        self.range(level).start
    }

    /// This method returns a range of color that each log level will
    /// use.  The provided color range defines the bounds for linear gradients,
    /// and can be used to calculate solid colors, if desired.
    fn range(&self, level: Level) -> RgbRange {
        match level {
            Level::Trace => RgbRange {
                start: Rgb {
                    r: 245,
                    g: 1,
                    b: 167,
                },
                end: Rgb {
                    r: 245,
                    g: 121,
                    b: 167,
                },
            },
            Level::Debug => RgbRange {
                start: Color::Cyan.value(),
                end: Rgb {
                    r: 162,
                    g: 187,
                    b: 214,
                },
            },
            Level::Info => RgbRange {
                start: Color::Green.value(),
                end: Rgb {
                    r: 209,
                    g: 235,
                    b: 165,
                },
            },
            Level::Warn => RgbRange {
                start: Color::Yellow.value(),
                end: Rgb {
                    r: 214,
                    g: 202,
                    b: 169,
                },
            },
            Level::Error => RgbRange {
                start: Color::Red.value(),
                end: Rgb {
                    r: 245,
                    g: 171,
                    b: 171,
                },
            },
        }
    }
}

fn main() {
    // setup logger
    let config = Config {
        level: LevelFilter::Trace,
        theme: Box::new(MyTheme {}),
        color_format: Some(ColorFormat::InlineGradient(20)),
        ..Default::default()
    };
    DiscoLogger::with_config(config).init().unwrap();

    // log away!
    log::trace!("I looked forward to making a crate");
    log::debug!("Where colors in logs annotate,");
    log::info!("But encountered a yak");
    log::warn!("And couldn't look back");
    log::error!("'til the hair on that yak did abate");
}
