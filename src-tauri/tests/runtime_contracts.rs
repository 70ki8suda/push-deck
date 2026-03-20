use push_deck::app_state::{AppState, RuntimeCapabilities, RuntimeState, ShortcutCapabilityState};
use push_deck::display::{DisplayAdapter, DisplayFrame, DisplayTarget, NoopDisplayAdapter};
use push_deck::events::{emit_runtime_event, RuntimeEvent, RUNTIME_EVENT_NAME};
use serde_json::json;
use std::sync::mpsc::channel;
use tauri::Listener;

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
fn runtime_event_helper_emits_on_the_shared_channel() {
    assert_eq!(RUNTIME_EVENT_NAME, "push-deck:runtime-event");

    let app = tauri::test::mock_app();
    let (tx, rx) = channel();
    let _listener_id = app.listen_any(RUNTIME_EVENT_NAME, move |event| {
        tx.send(event.payload().to_string()).expect("listener should send payload");
    });

    let event = RuntimeEvent::PadPressed {
        pad_id: "r0c0".to_string(),
    };

    emit_runtime_event(&app, event.clone()).expect("emit should succeed");

    let payload = rx.recv().expect("listener should receive payload");
    assert_eq!(payload, serde_json::to_string(&event).expect("payload should serialize"));
}

#[test]
fn noop_display_adapter_accepts_display_frames_as_async_operations() {
    tauri::async_runtime::block_on(async {
        let mut adapter = NoopDisplayAdapter::default();
        let frame = DisplayFrame {
            target: DisplayTarget::Main,
            payload: json!({
                "message": "hello"
            }),
        };

        adapter
            .connect()
            .await
            .expect("connect should succeed");
        adapter.render(frame).await.expect("render should succeed");
        adapter.clear().await.expect("clear should succeed");
        adapter
            .disconnect()
            .await
            .expect("disconnect should succeed");
    });
}
