use crate::Rgb;

#[allow(dead_code)]
pub enum Color {
    DarkMagenta,
    Magenta,
    DarkPink,
    Pink,
    DarkCyan,
    Cyan,
    DarkBlue,
    Blue,
    DarkGreen,
    Green,
    DarkYellow,
    Yellow,
    DarkOrange,
    Orange,
    DarkRed,
    Red,
}

impl Color {
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
