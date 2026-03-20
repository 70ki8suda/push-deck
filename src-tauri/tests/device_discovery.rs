use push_deck::app_state::{AppState, DeviceConnectionState, DeviceEndpointDescriptor};
use push_deck::device::discovery::{
    discover_push_device, emit_discovery_state, DeviceDiscoveryError, DeviceDiscoverySource,
    PushDeviceService,
};
use push_deck::events::RUNTIME_EVENT_NAME;
use serde_json::json;
use std::sync::mpsc::channel;
use std::time::Duration;
use tauri::Listener;

#[test]
fn no_device_found_reports_waiting_for_device() {
    let source = TestDiscoverySource::new(Ok(vec![]));
    let mut service = PushDeviceService::new(source);

    let result = service.discover().expect("discovery should succeed");

    assert_eq!(result.app_state, AppState::WaitingForDevice);
    assert_eq!(result.connection, DeviceConnectionState::WaitingForDevice);
    assert!(service.active_endpoint().is_none());
}

#[test]
fn one_push_three_found_is_bound_with_stable_endpoint_identity() {
    let endpoint = DeviceEndpointDescriptor::push_3("endpoint-123", "Ableton Push 3");
    let source = TestDiscoverySource::new(Ok(vec![endpoint.clone()]));
    let mut service = PushDeviceService::new(source);

    let result = service.discover().expect("discovery should succeed");

    assert_eq!(result.app_state, AppState::Ready);
    assert_eq!(
        result.connection,
        DeviceConnectionState::Connected {
            endpoint: endpoint.clone()
        }
    );
    assert_eq!(service.active_endpoint(), Some(&endpoint));
}

#[test]
fn multiple_candidate_devices_bind_the_first_push_three_match() {
    let expected_endpoint = DeviceEndpointDescriptor::push_3("endpoint-2", "First Push 3");
    let source = TestDiscoverySource::new(Ok(vec![
        DeviceEndpointDescriptor::other("endpoint-1", "Generic MIDI"),
        expected_endpoint.clone(),
        DeviceEndpointDescriptor::push_3("endpoint-3", "Second Push 3"),
    ]));
    let mut service = PushDeviceService::new(source);

    let result = service.discover().expect("discovery should succeed");

    assert_eq!(
        result.connection,
        DeviceConnectionState::Connected {
            endpoint: expected_endpoint.clone()
        }
    );
    assert_eq!(service.active_endpoint(), Some(&expected_endpoint));
}

#[test]
fn backend_discovery_error_is_returned_verbatim() {
    let source = TestDiscoverySource::new(Err(DeviceDiscoveryError::backend(
        "midi backend unavailable",
    )));
    let mut service = PushDeviceService::new(source);

    let error = service.discover().expect_err("backend error should bubble up");

    assert_eq!(
        error,
        DeviceDiscoveryError::backend("midi backend unavailable")
    );
}

#[test]
fn discovery_state_is_emitted_as_device_owned_runtime_event_only() {
    let app = tauri::test::mock_app();
    let (tx, rx) = channel();
    let _listener_id = app.listen_any(RUNTIME_EVENT_NAME, move |event| {
        tx.send(event.payload().to_string())
            .expect("listener should capture payload");
    });

    let state = discover_push_device(vec![DeviceEndpointDescriptor::push_3(
        "endpoint-123",
        "Ableton Push 3",
    )]);
    emit_discovery_state(&app, &state).expect("emission should succeed");

    let payload: serde_json::Value = rx
        .recv()
        .expect("device connection event")
        .parse()
        .expect("device connection event json");

    assert_eq!(
        payload,
        json!({
            "type": "device_connection_changed",
            "connected": true,
            "device_name": "Ableton Push 3"
        })
    );
    assert!(
        rx.recv_timeout(Duration::from_millis(100)).is_err(),
        "discovery should not emit unrelated runtime state"
    );
}

#[derive(Clone)]
struct TestDiscoverySource {
    response: Result<Vec<DeviceEndpointDescriptor>, DeviceDiscoveryError>,
}

impl TestDiscoverySource {
    fn new(response: Result<Vec<DeviceEndpointDescriptor>, DeviceDiscoveryError>) -> Self {
        Self { response }
    }
}

impl DeviceDiscoverySource for TestDiscoverySource {
    fn discover_devices(&self) -> Result<Vec<DeviceEndpointDescriptor>, DeviceDiscoveryError> {
        self.response.clone()
    }
}
