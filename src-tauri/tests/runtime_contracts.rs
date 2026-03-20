use push_deck::app_state::{AppState, RuntimeCapabilities, RuntimeState, ShortcutCapabilityState};
use push_deck::display::{DisplayAdapter, DisplayFrame, DisplayTarget, NoopDisplayAdapter};
use push_deck::events::RuntimeEvent;
use serde_json::json;

#[test]
fn runtime_state_serializes_with_app_state_and_capabilities() {
    let state = RuntimeState {
        app_state: AppState::Ready,
        capabilities: RuntimeCapabilities {
            shortcut: ShortcutCapabilityState::Unavailable,
        },
    };

    let serialized = serde_json::to_value(state).expect("state should serialize");

    assert_eq!(
        serialized,
        json!({
            "app_state": "ready",
            "capabilities": {
                "shortcut": "unavailable"
            }
        })
    );
}

#[test]
fn runtime_event_serializes_with_tagged_payloads() {
    let event = RuntimeEvent::PadPressed {
        pad_id: "r0c0".to_string(),
    };

    let serialized = serde_json::to_value(event).expect("event should serialize");

    assert_eq!(
        serialized,
        json!({
            "type": "pad_pressed",
            "pad_id": "r0c0"
        })
    );
}

#[test]
fn noop_display_adapter_accepts_display_frames_without_side_effects() {
    let mut adapter = NoopDisplayAdapter::default();
    let frame = DisplayFrame {
        target: DisplayTarget::Main,
        payload: json!({
            "message": "hello"
        }),
    };

    adapter.connect().expect("connect should succeed");
    adapter.render(frame).expect("render should succeed");
    adapter.clear().expect("clear should succeed");
    adapter.disconnect().expect("disconnect should succeed");
}
