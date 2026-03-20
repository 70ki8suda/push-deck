use push_deck::app_state::{AppState, DeviceConnectionState};
use push_deck::device::discovery::{
    discover_push_device, emit_discovery_state, DeviceCandidate, DeviceDiscoverySource,
    PushDeviceService,
};
use push_deck::events::RUNTIME_EVENT_NAME;
use serde_json::json;
use std::sync::mpsc::channel;
use tauri::Listener;

#[test]
fn no_device_found_reports_waiting_for_device() {
    let source = TestDiscoverySource::new(vec![]);
    let mut service = PushDeviceService::new(source);

    let result = service.discover().expect("discovery should succeed");

    assert_eq!(result.app_state, AppState::WaitingForDevice);
    assert_eq!(
        result.device_state,
        DeviceConnectionState::WaitingForDevice
    );
    assert!(result.active_device.is_none());
}

#[test]
fn one_push_three_found_is_bound() {
    let source = TestDiscoverySource::new(vec![DeviceCandidate::push_3("Ableton Push 3")]);
    let mut service = PushDeviceService::new(source);

    let result = service.discover().expect("discovery should succeed");

    assert_eq!(result.app_state, AppState::Ready);
    assert_eq!(
        result.device_state,
        DeviceConnectionState::Connected {
            device_name: "Ableton Push 3".to_string()
        }
    );
    assert_eq!(
        result.active_device.expect("active device"),
        "Ableton Push 3"
    );
}

#[test]
fn multiple_candidate_devices_bind_the_first_push_three_match() {
    let source = TestDiscoverySource::new(vec![
        DeviceCandidate::other("Generic MIDI"),
        DeviceCandidate::push_3("First Push 3"),
        DeviceCandidate::push_3("Second Push 3"),
    ]);
    let mut service = PushDeviceService::new(source);

    let result = service.discover().expect("discovery should succeed");

    assert_eq!(
        result.device_state,
        DeviceConnectionState::Connected {
            device_name: "First Push 3".to_string()
        }
    );
    assert_eq!(
        result.active_device.expect("active device"),
        "First Push 3"
    );
}

#[test]
fn discovery_state_is_emitted_as_runtime_events() {
    let app = tauri::test::mock_app();
    let (tx, rx) = channel();
    let _listener_id = app.listen_any(RUNTIME_EVENT_NAME, move |event| {
        tx.send(event.payload().to_string())
            .expect("listener should capture payload");
    });

    let state = discover_push_device(vec![DeviceCandidate::push_3("Ableton Push 3")]);
    emit_discovery_state(&app, &state).expect("emission should succeed");

    let mut payloads: Vec<serde_json::Value> = vec![
        rx.recv()
            .expect("state event")
            .parse()
            .expect("state event json"),
        rx.recv()
            .expect("connection event")
            .parse()
            .expect("connection event json"),
    ];
    payloads.sort_by(|left, right| left.to_string().cmp(&right.to_string()));

    assert_eq!(
        payloads,
        vec![
            json!({
                "type": "device_connection_changed",
                "connected": true,
                "device_name": "Ableton Push 3"
            }),
            json!({
                "type": "state_changed",
                "state": {
                    "app_state": "ready",
                    "capabilities": {
                        "shortcut": "unavailable"
                    }
                }
            })
        ]
    );
}

#[derive(Clone)]
struct TestDiscoverySource {
    candidates: Vec<DeviceCandidate>,
}

impl TestDiscoverySource {
    fn new(candidates: Vec<DeviceCandidate>) -> Self {
        Self { candidates }
    }
}

impl DeviceDiscoverySource for TestDiscoverySource {
    fn discover_devices(&self) -> Vec<DeviceCandidate> {
        self.candidates.clone()
    }
}
