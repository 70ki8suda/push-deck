use crate::config::schema::PadColorId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Push3Color {
    Off = 0,
    White = 1,
    Red = 2,
    Orange = 3,
    Yellow = 4,
    Green = 5,
    Cyan = 6,
    Blue = 7,
    Purple = 8,
    Pink = 9,
}

impl Push3Color {
    pub fn device_value(self) -> u8 {
        self as u8
    }
}

pub fn map_pad_color_id(color_id: PadColorId) -> Push3Color {
    match color_id {
        PadColorId::Off => Push3Color::Off,
        PadColorId::White => Push3Color::White,
        PadColorId::Red => Push3Color::Red,
        PadColorId::Orange => Push3Color::Orange,
        PadColorId::Yellow => Push3Color::Yellow,
        PadColorId::Green => Push3Color::Green,
        PadColorId::Cyan => Push3Color::Cyan,
        PadColorId::Blue => Push3Color::Blue,
        PadColorId::Purple => Push3Color::Purple,
        PadColorId::Pink => Push3Color::Pink,
    }
}
