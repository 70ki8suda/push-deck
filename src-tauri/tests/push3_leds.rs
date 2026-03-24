use push_deck::config::schema::PadColorId;
use push_deck::device::colors::{map_pad_color_id, map_pad_color_rgb};
use push_deck::device::output::{
    encode_led_command_word, encode_pad_rgb_sysex, render_config_pad_led_commands,
    render_config_pad_rgb_commands, Push3PadRgbCommand,
};
use push_deck::device::push3::{
    coordinate_for_pad_id, coordinate_for_transport_pad_index, decode_transport_pad_input,
    pad_id_for_coordinate, render_pad_leds, transport_pad_index_for_coordinate,
    DecodedPadInputMessage, Push3PadCoordinate, Push3PadLed, Push3TransportLedCommand,
    Push3TransportPadIndex,
    Push3TransportPadInputMessage,
};
use push_deck::config::schema::{Config, PadAction};

#[test]
fn every_pad_color_id_maps_to_a_device_color_value() {
    let cases = [
        (PadColorId::Off, 0),
        (PadColorId::White, 3),
        (PadColorId::Peach, 8),
        (PadColorId::Coral, 4),
        (PadColorId::Red, 5),
        (PadColorId::Orange, 9),
        (PadColorId::Amber, 12),
        (PadColorId::Yellow, 13),
        (PadColorId::Lime, 16),
        (PadColorId::Chartreuse, 17),
        (PadColorId::Green, 21),
        (PadColorId::Mint, 29),
        (PadColorId::Teal, 33),
        (PadColorId::Cyan, 37),
        (PadColorId::Sky, 41),
        (PadColorId::Blue, 45),
        (PadColorId::Indigo, 48),
        (PadColorId::Purple, 49),
        (PadColorId::Magenta, 52),
        (PadColorId::Rose, 56),
        (PadColorId::Pink, 57),
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
fn every_pad_color_id_maps_to_a_device_rgb_value() {
    let cases = [
        (PadColorId::Off, (0, 0, 0)),
        (PadColorId::White, (255, 255, 255)),
        (PadColorId::Peach, (255, 189, 108)),
        (PadColorId::Coral, (255, 76, 76)),
        (PadColorId::Red, (255, 0, 0)),
        (PadColorId::Orange, (255, 84, 0)),
        (PadColorId::Amber, (255, 255, 76)),
        (PadColorId::Yellow, (255, 255, 0)),
        (PadColorId::Lime, (136, 255, 76)),
        (PadColorId::Chartreuse, (84, 255, 0)),
        (PadColorId::Green, (0, 255, 0)),
        (PadColorId::Mint, (0, 255, 85)),
        (PadColorId::Teal, (0, 255, 153)),
        (PadColorId::Cyan, (0, 169, 255)),
        (PadColorId::Sky, (0, 85, 255)),
        (PadColorId::Blue, (0, 0, 255)),
        (PadColorId::Indigo, (135, 76, 255)),
        (PadColorId::Purple, (84, 0, 255)),
        (PadColorId::Magenta, (255, 76, 255)),
        (PadColorId::Rose, (255, 76, 135)),
        (PadColorId::Pink, (255, 0, 84)),
    ];

    for (color_id, expected_rgb) in cases {
        let mapped = map_pad_color_rgb(color_id);
        assert_eq!(
            (mapped.red, mapped.green, mapped.blue),
            expected_rgb,
            "unexpected rgb value for {color_id:?}"
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

            assert_eq!(
                decoded,
                DecodedPadInputMessage::PadPressed {
                    pad_id: expected_pad_id,
                    velocity: 127,
                }
            );
        }
    }
}

#[test]
fn inbound_transport_pad_releases_resolve_to_the_correct_pad_id() {
    for row in 0..8 {
        for column in 0..8 {
            let expected_pad_id = format!("r{row}c{column}");
            let coordinate = Push3PadCoordinate { row, column };
            let transport_index = transport_pad_index_for_coordinate(coordinate).expect("index");
            let message = Push3TransportPadInputMessage::PadReleased { transport_index };

            let decoded = decode_transport_pad_input(message).expect("pad release should decode");

            assert_eq!(
                decoded,
                DecodedPadInputMessage::PadReleased {
                    pad_id: expected_pad_id,
                }
            );
        }
    }
}

#[test]
fn push3_user_port_pad_notes_follow_the_observed_corner_mapping() {
    assert_eq!(
        transport_pad_index_for_coordinate(Push3PadCoordinate { row: 0, column: 0 }),
        Some(Push3TransportPadIndex(0x5C))
    );
    assert_eq!(
        transport_pad_index_for_coordinate(Push3PadCoordinate { row: 7, column: 7 }),
        Some(Push3TransportPadIndex(0x2B))
    );
    assert_eq!(
        transport_pad_index_for_coordinate(Push3PadCoordinate { row: 3, column: 3 }),
        Some(Push3TransportPadIndex(0x47))
    );
    assert_eq!(
        coordinate_for_transport_pad_index(Push3TransportPadIndex(0x5C)),
        Some(Push3PadCoordinate { row: 0, column: 0 })
    );
    assert_eq!(
        coordinate_for_transport_pad_index(Push3TransportPadIndex(0x2B)),
        Some(Push3PadCoordinate { row: 7, column: 7 })
    );
}

#[test]
fn render_narrow_led_entries_to_transport_commands_using_the_same_note_mapping() {
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
            transport_index: Push3TransportPadIndex(0x5C),
            color_value: 21,
        }
    );
    assert_eq!(
        frame[63],
        Push3TransportLedCommand {
            transport_index: Push3TransportPadIndex(0x2B),
            color_value: 57,
        }
    );
}

#[test]
fn render_config_pad_led_commands_projects_the_active_profile_colors_to_all_64_pads() {
    let mut config = Config::default();
    config.profiles[0].pads[0].color = PadColorId::Green;
    config.profiles[0].pads[63].color = PadColorId::Pink;
    config.profiles[0].pads[0].action =
        PadAction::launch_or_focus_app("com.apple.Terminal", "Terminal");

    let frame = render_config_pad_led_commands(&config);

    assert_eq!(frame.len(), 64);
    assert_eq!(
        frame[0],
        Push3TransportLedCommand {
            transport_index: Push3TransportPadIndex(0x5C),
            color_value: 21,
        }
    );
    assert_eq!(
        frame[63],
        Push3TransportLedCommand {
            transport_index: Push3TransportPadIndex(0x2B),
            color_value: 57,
        }
    );
}

#[test]
fn encode_led_command_word_uses_midi10_note_on_words() {
    assert_eq!(
        encode_led_command_word(Push3TransportLedCommand {
            transport_index: Push3TransportPadIndex(0x5C),
            color_value: 21,
        }),
        0x2090_5C15
    );
}

#[test]
fn render_config_pad_rgb_commands_projects_the_active_profile_colors_to_push_pad_indices() {
    let mut config = Config::default();
    config.profiles[0].pads[0].color = PadColorId::Red;
    config.profiles[0].pads[63].color = PadColorId::Blue;

    let commands = render_config_pad_rgb_commands(&config);

    assert_eq!(commands.len(), 64);
    assert_eq!(
        commands[0],
        Push3PadRgbCommand {
            pad_index: 56,
            red: 255,
            green: 0,
            blue: 0,
        }
    );
    assert_eq!(
        commands[63],
        Push3PadRgbCommand {
            pad_index: 7,
            red: 0,
            green: 0,
            blue: 255,
        }
    );
}

#[test]
fn encode_pad_rgb_sysex_uses_push_user_port_rgb_format() {
    assert_eq!(
        encode_pad_rgb_sysex(Push3PadRgbCommand {
            pad_index: 56,
            red: 255,
            green: 0,
            blue: 84,
        }),
        vec![
            0xF0, 0x47, 0x7F, 0x15, 0x04, 0x00, 0x08, 56, 0x00, 0x0F, 0x0F, 0x00, 0x00,
            0x05, 0x04, 0xF7,
        ]
    );
}
