use push_deck::device::{
    decode_midi1_channel_voice_word, decode_push_mode_message, emit_decoded_pad_input_event,
    select_push3_user_port_source, Push3InputSourceDescriptor,
};
use push_deck::device::push3::DecodedPadInputMessage;
use push_deck::device::PushModeEvent;
use push_deck::events::RUNTIME_EVENT_NAME;
use serde_json::json;
use std::sync::mpsc::channel;
use tauri::Listener;

#[test]
fn selects_the_push3_user_port_source_when_present() {
    let selected = select_push3_user_port_source(&[
        Push3InputSourceDescriptor {
            unique_id: 1,
            display_name: "Ableton Push 3 Live Port".to_string(),
        },
        Push3InputSourceDescriptor {
            unique_id: 2,
            display_name: "Ableton Push 3 User Port".to_string(),
        },
    ]);

    assert_eq!(
        selected,
        Some(Push3InputSourceDescriptor {
            unique_id: 2,
            display_name: "Ableton Push 3 User Port".to_string(),
        })
    );
}

#[test]
fn returns_none_when_no_push3_user_port_source_exists() {
    let selected = select_push3_user_port_source(&[]);

    assert_eq!(selected, None);
}

#[test]
fn decodes_midi10_note_on_and_note_off_words_into_pad_messages() {
    assert_eq!(
        decode_midi1_channel_voice_word(0x20905c46),
        Some(DecodedPadInputMessage::PadPressed {
            pad_id: "r0c0".to_string(),
            velocity: 0x46,
        })
    );
    assert_eq!(
        decode_midi1_channel_voice_word(0x20805c40),
        Some(DecodedPadInputMessage::PadReleased {
            pad_id: "r0c0".to_string(),
        })
    );
    assert_eq!(decode_midi1_channel_voice_word(0x10f00000), None);
}

#[test]
fn decodes_push_user_mode_cc_and_sysex_messages() {
    assert_eq!(
        decode_push_mode_message(&[0xB0, 0x3B, 0x7F]),
        Some(PushModeEvent::UserModeButtonPressed)
    );
    assert_eq!(
        decode_push_mode_message(&[0xB0, 0x3B, 0x00]),
        Some(PushModeEvent::UserModeButtonReleased)
    );
    assert_eq!(
        decode_push_mode_message(&[0xF0, 0x00, 0x21, 0x1D, 0x01, 0x01, 0x0A, 0x01, 0xF7]),
        Some(PushModeEvent::UserModeEntered)
    );
    assert_eq!(
        decode_push_mode_message(&[0xF0, 0x00, 0x21, 0x1D, 0x01, 0x01, 0x0A, 0x00, 0xF7]),
        Some(PushModeEvent::UserModeExited)
    );
}

#[test]
fn ignores_unrelated_push_mode_messages() {
    assert_eq!(decode_push_mode_message(&[0x90, 0x3B, 0x7F]), None);
    assert_eq!(
        decode_push_mode_message(&[0xF0, 0x00, 0x21, 0x1D, 0x01, 0x01, 0x0B, 0x01, 0xF7]),
        None
    );
}

#[test]
fn emits_runtime_pad_events_for_decoded_messages() {
    let app = tauri::test::mock_app();
    let (tx, rx) = channel();
    let _listener_id = app.listen_any(RUNTIME_EVENT_NAME, move |event| {
        tx.send(event.payload().to_string())
            .expect("listener should capture payload");
    });

    emit_decoded_pad_input_event(
        &app,
        DecodedPadInputMessage::PadPressed {
            pad_id: "r0c0".to_string(),
            velocity: 0x40,
        },
    )
    .expect("pad press should emit");
    emit_decoded_pad_input_event(
        &app,
        DecodedPadInputMessage::PadReleased {
            pad_id: "r0c0".to_string(),
        },
    )
    .expect("pad release should emit");

    let pressed: serde_json::Value = rx.recv().expect("pressed event").parse().expect("json");
    let released: serde_json::Value = rx.recv().expect("released event").parse().expect("json");

    assert_eq!(
        pressed,
        json!({
            "type": "pad_pressed",
            "pad_id": "r0c0"
        })
    );
    assert_eq!(
        released,
        json!({
            "type": "pad_released",
            "pad_id": "r0c0"
        })
    );
}

#[cfg(not(target_os = "macos"))]
#[test]
fn startup_subscription_returns_not_connected_when_user_port_is_unavailable() {
    let app = tauri::test::mock_app();

    let subscription = push_deck::device::subscribe_push3_user_port_runtime_events(&app.handle())
        .expect("non-macos builds should skip CoreMIDI input subscription");

    assert!(matches!(
        subscription,
        push_deck::device::Push3InputSubscription::NotConnected
    ));
}
