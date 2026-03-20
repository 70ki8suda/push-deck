use push_deck::config::schema::{PadAction, PadBinding, PadColorId};
use push_deck::device::colors::map_pad_color_id;
use push_deck::device::push3::{
    coordinate_for_pad_id, decode_pad_input, pad_id_for_coordinate, render_pad_grid,
    Push3LedState, Push3PadCoordinate, Push3PadInputMessage,
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
fn pad_ids_r0c0_through_r7c7_map_to_expected_device_coordinates_and_back() {
    for row in 0..8 {
        for column in 0..8 {
            let expected_pad_id = format!("r{row}c{column}");
            let coordinate = coordinate_for_pad_id(&expected_pad_id).expect("coordinate");

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
        }
    }
}

#[test]
fn inbound_pad_messages_resolve_to_the_correct_pad_id() {
    for row in 0..8 {
        for column in 0..8 {
            let expected_pad_id = format!("r{row}c{column}");
            let message = Push3PadInputMessage::PadPressed {
                coordinate: Push3PadCoordinate { row, column },
                velocity: 127,
            };

            let decoded = decode_pad_input(message).expect("pad press should decode");

            assert_eq!(decoded.pad_id, expected_pad_id);
            assert_eq!(decoded.velocity, 127);
        }
    }
}

#[test]
fn render_full_grid_outputs_64_led_states_in_row_major_order() {
    let frame = render_pad_grid(&[
        PadBinding {
            pad_id: "r0c0".to_string(),
            label: "Launch".to_string(),
            color: PadColorId::Green,
            action: PadAction::Unassigned,
        },
        PadBinding {
            pad_id: "r7c7".to_string(),
            label: "Focus".to_string(),
            color: PadColorId::Pink,
            action: PadAction::Unassigned,
        },
    ]);

    assert_eq!(frame.len(), 64);
    assert_eq!(
        frame[0],
        Push3LedState {
            coordinate: Push3PadCoordinate { row: 0, column: 0 },
            color_value: 5,
        }
    );
    assert_eq!(
        frame[63],
        Push3LedState {
            coordinate: Push3PadCoordinate { row: 7, column: 7 },
            color_value: 9,
        }
    );
}
