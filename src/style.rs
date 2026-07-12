#![allow(unused)]

use std::fmt;

#[cfg(feature = "fancy")]
use crate::ParseError;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Style(u16);

const EMPTY: Style = Style(0);

const BOLD: u16 = 1 << 0;
const ITALIC: u16 = 1 << 1;
const UNDERLINE: u16 = 1 << 2;
const DIM: u16 = 1 << 3;

const FG_MASK: u16 = 0b000001111100000; // Bits 5-9
const BG_MASK: u16 = 0b111110000000000; // Bits 10-14
const FG_SHIFT: u16 = 5;
const BG_SHIFT: u16 = 10;

impl Style {
    pub const fn new() -> Self {
        EMPTY
    }

    pub const fn bold(mut self) -> Self {
        self.0 |= BOLD;
        self
    }

    pub const fn italic(mut self) -> Self {
        self.0 |= ITALIC;
        self
    }

    pub const fn underline(mut self) -> Self {
        self.0 |= UNDERLINE;
        self
    }

    pub const fn dim(mut self) -> Self {
        self.0 |= DIM;
        self
    }

    pub const fn fg(mut self, color: Color) -> Self {
        self.0 = (self.0 & !FG_MASK) | ((color as u16) << FG_SHIFT);
        self
    }

    pub const fn bg(mut self, color: Color) -> Self {
        self.0 = (self.0 & !BG_MASK) | ((color as u16) << BG_SHIFT);
        self
    }

    pub const fn is_bold(self) -> bool {
        self.0 & BOLD != 0
    }

    pub const fn is_italic(self) -> bool {
        self.0 & ITALIC != 0
    }

    pub const fn is_underline(self) -> bool {
        self.0 & UNDERLINE != 0
    }

    pub const fn is_dim(self) -> bool {
        self.0 & DIM != 0
    }

    pub const fn fg_color(self) -> Color {
        let raw = (self.0 & FG_MASK) >> FG_SHIFT;
        Color::from_u16(raw)
    }

    pub const fn bg_color(self) -> Color {
        let raw = (self.0 & BG_MASK) >> BG_SHIFT;
        Color::from_u16(raw)
    }

    pub const fn merge(self, other: Style) -> Style {
        let attrs = (self.0 | other.0) & 0x000F;

        let fg = if other.0 & FG_MASK != 0 {
            other.0 & FG_MASK
        } else {
            self.0 & FG_MASK
        };

        let bg = if other.0 & BG_MASK != 0 {
            other.0 & BG_MASK
        } else {
            self.0 & BG_MASK
        };

        Style(attrs | fg | bg)
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn write_ansi_prefix(&self, f: &mut impl fmt::Write) -> fmt::Result {
        if self.is_empty() {
            return Ok(());
        }

        write!(f, "\x1b[")?;
        let mut first = true;

        if self.is_bold() {
            write!(f, "1")?;
            first = false;
        }
        if self.is_italic() {
            write!(f, "{}{}", if first { "" } else { ";" }, "3")?;
            first = false;
        }
        if self.is_underline() {
            write!(f, "{}{}", if first { "" } else { ";" }, "4")?;
            first = false;
        }
        if self.is_dim() {
            write!(f, "{}{}", if first { "" } else { ";" }, "2")?;
            first = false;
        }

        let fg = self.fg_color();
        let bg = self.bg_color();

        if fg != Color::None {
            let fg_code = match fg as u16 {
                0 => unreachable!(),
                1..=8 => 29 + fg as u16,
                _ => 81 + fg as u16, // bright colors
            };

            write!(f, "{}{}", if first { "" } else { ";" }, fg_code)?;
            first = false;
        }

        if bg != Color::None {
            let bg_code = match bg as u16 {
                0 => unreachable!(),
                1..=8 => 39 + bg as u16,
                _ => 91 + bg as u16, // Bright backgrounds map to 100-107
            };

            write!(f, "{}{}", if first { "" } else { ";" }, bg_code)?;
        }

        write!(f, "m")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Color {
    None = 0,
    Black = 1,
    Red = 2,
    Green = 3,
    Yellow = 4,
    Blue = 5,
    Magenta = 6,
    Cyan = 7,
    White = 8,
    BrightBlack = 9,
    BrightRed = 10,
    BrightGreen = 11,
    BrightYellow = 12,
    BrightBlue = 13,
    BrightMagenta = 14,
    BrightCyan = 15,
    BrightWhite = 16,
}

impl Color {
    pub const fn from_u16(v: u16) -> Color {
        use Color::*;
        match v {
            1 => Black,
            2 => Red,
            3 => Green,
            4 => Yellow,
            5 => Blue,
            6 => Magenta,
            7 => Cyan,
            8 => White,
            9 => BrightBlack,
            10 => BrightRed,
            11 => BrightGreen,
            12 => BrightYellow,
            13 => BrightBlue,
            14 => BrightMagenta,
            15 => BrightCyan,
            16 => BrightWhite,
            _ => Color::None,
        }
    }
}

#[cfg(feature = "fancy")]
pub fn parse_color(s: &str) -> Result<Color, ParseError> {
    Ok(match s {
        "red" => Color::Red,
        "green" => Color::Green,
        "blue" => Color::Blue,
        "yellow" => Color::Yellow,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "black" => Color::Black,

        _ => return Err(ParseError(format!("invalid color name: {s}"))),
    })
}
