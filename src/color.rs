//! Color related type definitions and constant values

use colored::{self, Color::TrueColor};

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

impl Into<colored::Color> for Rgb {
    /// Convert Rgb -> Color for easier use with string coloring
    fn into(self) -> colored::Color {
        TrueColor {
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

#[allow(dead_code)]
/// Constant predefined color values
///
/// Use of these color values is not necessary when
/// creating a theme, but they act as helpful aliases
/// for raw Rgb triplets
pub enum Color {
    /// dark magenta
    DarkMagenta,
    /// magenta
    Magenta,
    /// dark pink
    DarkPink,
    /// pink
    Pink,
    /// dark cyan
    DarkCyan,
    /// cyan
    Cyan,
    /// dark blue
    DarkBlue,
    /// blue
    Blue,
    /// dark green
    DarkGreen,
    /// green
    Green,
    /// dark yellow
    DarkYellow,
    /// yellow
    Yellow,
    /// dark orange
    DarkOrange,
    /// orange
    Orange,
    /// dark red
    DarkRed,
    /// red
    Red,
}

impl Color {
    /// Return the Rgb triplet associated with a color variant
    pub fn value(&self) -> Rgb {
        match self {
            Color::DarkMagenta => Rgb {
                r: 139,
                g: 0,
                b: 139,
            },
            Color::Magenta => Rgb {
                r: 255,
                g: 0,
                b: 255,
            },
            Color::DarkPink => Rgb {
                r: 149,
                g: 119,
                b: 149,
            },
            Color::Pink => Rgb {
                r: 227,
                g: 184,
                b: 227,
            },
            Color::DarkCyan => Rgb {
                r: 10,
                g: 144,
                b: 144,
            },
            Color::Cyan => Rgb {
                r: 20,
                g: 210,
                b: 210,
            },
            Color::DarkBlue => Rgb {
                r: 70,
                g: 75,
                b: 185,
            },
            Color::Blue => Rgb {
                r: 90,
                g: 100,
                b: 240,
            },
            Color::DarkGreen => Rgb {
                r: 70,
                g: 140,
                b: 10,
            },
            Color::Green => Rgb {
                r: 110,
                g: 220,
                b: 10,
            },
            Color::DarkYellow => Rgb {
                r: 170,
                g: 128,
                b: 0,
            },
            Color::Yellow => Rgb {
                r: 255,
                g: 185,
                b: 0,
            },
            Color::DarkOrange => Rgb {
                r: 255,
                g: 128,
                b: 0,
            },
            Color::Orange => Rgb {
                r: 250,
                g: 180,
                b: 110,
            },
            Color::DarkRed => Rgb {
                r: 200,
                g: 0,
                b: 10,
            },
            Color::Red => Rgb {
                r: 255,
                g: 60,
                b: 10,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_into_color_is_accurate() {
        let test_cases = vec![
            (Rgb { r: 0, g: 0, b: 0 }, TrueColor { r: 0, g: 0, b: 0 }),
            (
                Rgb {
                    r: 127,
                    g: 128,
                    b: 129,
                },
                TrueColor {
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
                TrueColor {
                    r: 255,
                    g: 255,
                    b: 255,
                },
            ),
        ];

        for (rgb, tc) in test_cases {
            let c: colored::Color = rgb.into();
            assert_eq!(c, tc);
        }
    }
}
