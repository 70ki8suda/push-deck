use push_deck::config::schema::PadColorId;
use push_deck::device::colors::map_pad_color_id;
use push_deck::device::push3::{
    coordinate_for_pad_id, coordinate_for_transport_pad_index, decode_transport_pad_input,
    pad_id_for_coordinate, render_pad_leds, transport_pad_index_for_coordinate,
    Push3PadCoordinate, Push3PadLed, Push3TransportLedCommand, Push3TransportPadIndex,
    Push3TransportPadInputMessage,
};

#[test]
fn every_pad_color_id_maps_to_a_device_color_value() {
    let cases = [
        (PadColorId::Off, 0),
        (PadColorId::White, 1),
        (PadColorId::Red, 2),
        (PadColorId::Orange, 3),
        (PadColorId::Yellow, 4),
        (PadColorId::Green, 5),
        (PadColorId::Cyan, 6),
        (PadColorId::Blue, 7),
        (PadColorId::Purple, 8),
        (PadColorId::Pink, 9),
    ];

    for (color_id, expected_value) in cases {
        let mapped = map_pad_color_id(color_id);
        assert_eq!(
            mapped.device_value(),
            expected_value,
            "unexpected device value for {color_id:?}"
        );
    }
}

#[test]
fn pad_ids_r0c0_through_r7c7_map_to_expected_device_coordinates_and_transport_indices() {
    for row in 0..8 {
        for column in 0..8 {
            let expected_pad_id = format!("r{row}c{column}");
            let coordinate = coordinate_for_pad_id(&expected_pad_id).expect("coordinate");
            let transport_index = transport_pad_index_for_coordinate(coordinate).expect("index");

            assert_eq!(
                coordinate,
                Push3PadCoordinate { row, column },
                "pad id {expected_pad_id} should resolve to the matching coordinate"
            );
            assert_eq!(
                pad_id_for_coordinate(coordinate).as_deref(),
                Some(expected_pad_id.as_str()),
                "coordinate {row},{column} should resolve to the matching pad id"
            );
            assert_eq!(
                coordinate_for_transport_pad_index(transport_index).expect("coordinate"),
                coordinate,
                "transport index should round-trip back to the same coordinate"
            );
        }
    }
}

#[test]
fn inbound_transport_pad_messages_resolve_to_the_correct_pad_id() {
    for row in 0..8 {
        for column in 0..8 {
            let expected_pad_id = format!("r{row}c{column}");
            let coordinate = Push3PadCoordinate { row, column };
            let transport_index = transport_pad_index_for_coordinate(coordinate).expect("index");
            let message = Push3TransportPadInputMessage::PadPressed {
                transport_index,
                velocity: 127,
            };

            let decoded = decode_transport_pad_input(message).expect("pad press should decode");

            assert_eq!(decoded.pad_id, expected_pad_id);
            assert_eq!(decoded.velocity, 127);
        }
    }
}

#[test]
fn render_narrow_led_entries_to_transport_commands_in_row_major_order() {
    let frame = render_pad_leds(&[
        Push3PadLed {
            pad_id: "r0c0".to_string(),
            color: map_pad_color_id(PadColorId::Green),
        },
        Push3PadLed {
            pad_id: "r7c7".to_string(),
            color: map_pad_color_id(PadColorId::Pink),
        },
    ]);

    assert_eq!(frame.len(), 64);
    assert_eq!(
        frame[0],
        Push3TransportLedCommand {
            transport_index: Push3TransportPadIndex(0),
            color_value: 5,
        }
    );
    assert_eq!(
        frame[63],
        Push3TransportLedCommand {
            transport_index: Push3TransportPadIndex(63),
            color_value: 9,
        }
    );
}
