use crate::config::schema::PadBinding;

use super::colors::map_pad_color_id;

pub const PAD_ROWS: u8 = 8;
pub const PAD_COLUMNS: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Push3PadCoordinate {
    pub row: u8,
    pub column: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Push3PadInputMessage {
    PadPressed {
        coordinate: Push3PadCoordinate,
        velocity: u8,
    },
    PadReleased {
        coordinate: Push3PadCoordinate,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedPadInput {
    pub pad_id: String,
    pub velocity: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Push3LedState {
    pub coordinate: Push3PadCoordinate,
    pub color_value: u8,
}

pub fn coordinate_for_pad_id(pad_id: &str) -> Option<Push3PadCoordinate> {
    let (row_part, column_part) = pad_id.strip_prefix('r')?.split_once('c')?;
    let row = row_part.parse::<u8>().ok()?;
    let column = column_part.parse::<u8>().ok()?;

    if row < PAD_ROWS && column < PAD_COLUMNS {
        Some(Push3PadCoordinate { row, column })
    } else {
        None
    }
}

pub fn pad_id_for_coordinate(coordinate: Push3PadCoordinate) -> Option<String> {
    if coordinate.row < PAD_ROWS && coordinate.column < PAD_COLUMNS {
        Some(format!("r{}c{}", coordinate.row, coordinate.column))
    } else {
        None
    }
}

pub fn decode_pad_input(message: Push3PadInputMessage) -> Option<DecodedPadInput> {
    match message {
        Push3PadInputMessage::PadPressed {
            coordinate,
            velocity,
        } => pad_id_for_coordinate(coordinate).map(|pad_id| DecodedPadInput { pad_id, velocity }),
        Push3PadInputMessage::PadReleased { .. } => None,
    }
}

pub fn render_pad_grid(bindings: &[PadBinding]) -> Vec<Push3LedState> {
    let mut frame = Vec::with_capacity((PAD_ROWS * PAD_COLUMNS) as usize);

    for row in 0..PAD_ROWS {
        for column in 0..PAD_COLUMNS {
            let coordinate = Push3PadCoordinate { row, column };
            let color_value = bindings
                .iter()
                .find(|binding| binding.pad_id == format!("r{}c{}", row, column))
                .map(|binding| map_pad_color_id(binding.color).device_value())
                .unwrap_or(0);

            frame.push(Push3LedState {
                coordinate,
                color_value,
            });
        }
    }

    frame
}
