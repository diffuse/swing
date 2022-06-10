use crate::Rgb;

#[allow(dead_code)]
pub enum Color {
    DarkPink,
    Pink,
    DarkMagenta,
    Magenta,
    DarkCyan,
    Cyan,
    DarkGreen,
    Green,
    DarkOrange,
    Orange,
    DarkRed,
    Red,
}

impl Color {
    pub fn value(&self) -> Rgb {
        match self {
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
