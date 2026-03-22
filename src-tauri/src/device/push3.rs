use super::colors::Push3Color;

pub const PAD_ROWS: u8 = 8;
pub const PAD_COLUMNS: u8 = 8;
pub const PAD_COUNT: usize = (PAD_ROWS as usize) * (PAD_COLUMNS as usize);
const PAD_NOTE_BASE: u8 = 0x24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Push3PadCoordinate {
    pub row: u8,
    pub column: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Push3TransportPadIndex(pub u8);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Push3TransportPadInputMessage {
    PadPressed {
        transport_index: Push3TransportPadIndex,
        velocity: u8,
    },
    PadReleased {
        transport_index: Push3TransportPadIndex,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodedPadInputMessage {
    PadPressed { pad_id: String, velocity: u8 },
    PadReleased { pad_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Push3PadLed {
    pub pad_id: String,
    pub color: Push3Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Push3TransportLedCommand {
    pub transport_index: Push3TransportPadIndex,
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

pub fn transport_pad_index_for_coordinate(
    coordinate: Push3PadCoordinate,
) -> Option<Push3TransportPadIndex> {
    if coordinate.row < PAD_ROWS && coordinate.column < PAD_COLUMNS {
        Some(Push3TransportPadIndex(
            PAD_NOTE_BASE + (PAD_ROWS - 1 - coordinate.row) * PAD_COLUMNS + coordinate.column,
        ))
    } else {
        None
    }
}

pub fn coordinate_for_transport_pad_index(
    transport_index: Push3TransportPadIndex,
) -> Option<Push3PadCoordinate> {
    let note_offset = transport_index.0.checked_sub(PAD_NOTE_BASE)?;

    if note_offset < PAD_COUNT as u8 {
        Some(Push3PadCoordinate {
            row: PAD_ROWS - 1 - (note_offset / PAD_COLUMNS),
            column: note_offset % PAD_COLUMNS,
        })
    } else {
        None
    }
}

pub fn decode_transport_pad_input(
    message: Push3TransportPadInputMessage,
) -> Option<DecodedPadInputMessage> {
    match message {
        Push3TransportPadInputMessage::PadPressed {
            transport_index,
            velocity,
        } => coordinate_for_transport_pad_index(transport_index)
            .and_then(|coordinate| pad_id_for_coordinate(coordinate))
            .map(|pad_id| DecodedPadInputMessage::PadPressed { pad_id, velocity }),
        Push3TransportPadInputMessage::PadReleased { transport_index } => {
            coordinate_for_transport_pad_index(transport_index)
                .and_then(|coordinate| pad_id_for_coordinate(coordinate))
                .map(|pad_id| DecodedPadInputMessage::PadReleased { pad_id })
        }
    }
}

pub fn render_pad_leds(leds: &[Push3PadLed]) -> Vec<Push3TransportLedCommand> {
    let mut frame = Vec::with_capacity(PAD_COUNT);

    for row in 0..PAD_ROWS {
        for column in 0..PAD_COLUMNS {
            let coordinate = Push3PadCoordinate { row, column };
            let transport_index = transport_pad_index_for_coordinate(coordinate)
                .expect("all logical pad coordinates map to a transport index");
            let color_value = leds
                .iter()
                .find(|led| led.pad_id == format!("r{}c{}", row, column))
                .map(|led| led.color.device_value())
                .unwrap_or(0);

            frame.push(Push3TransportLedCommand {
                transport_index,
                color_value,
            });
        }
    }

    frame
}
