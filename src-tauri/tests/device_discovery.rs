use push_deck::app_state::{
    AppState, DeviceConnectionState, DeviceEndpointDescriptor, RuntimeCapabilities, RuntimeState,
    ShortcutCapabilityState,
};
use push_deck::device::discovery::{
    discover_push_device, emit_discovery_state, DeviceDiscoveryError, DeviceDiscoverySource,
    PushDeviceService, SystemDiscoverySource,
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
fn user_port_is_preferred_over_other_push_three_ports() {
    let user_port = DeviceEndpointDescriptor::push_3("user-port", "Ableton Push 3 User Port");
    let source = TestDiscoverySource::new(Ok(vec![
        DeviceEndpointDescriptor::push_3("midi-port", "Ableton Push 3 MIDI Port"),
        user_port.clone(),
    ]));
    let mut service = PushDeviceService::new(source);

    let result = service.discover().expect("discovery should succeed");

    assert_eq!(
        result.connection,
        DeviceConnectionState::Connected {
            endpoint: user_port.clone()
        }
    );
    assert_eq!(service.active_endpoint(), Some(&user_port));
}

#[test]
fn backend_discovery_error_is_returned_verbatim() {
    let source = TestDiscoverySource::new(Err(DeviceDiscoveryError::backend(
        "midi backend unavailable",
    )));
    let mut service = PushDeviceService::new(source);

    let error = service
        .discover()
        .expect_err("backend error should bubble up");

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
    let runtime_state = RuntimeState {
        app_state: AppState::Starting,
        capabilities: RuntimeCapabilities {
            shortcut: ShortcutCapabilityState::Available,
        },
    };
    emit_discovery_state(&app, &state, &runtime_state).expect("emission should succeed");

    let state_payload: serde_json::Value = rx
        .recv()
        .expect("state event")
        .parse()
        .expect("state event json");
    let connection_payload: serde_json::Value = rx
        .recv()
        .expect("device connection event")
        .parse()
        .expect("device connection event json");

    assert_eq!(
        state_payload,
        json!({
            "type": "state_changed",
            "state": {
                "app_state": "ready",
                "capabilities": {
                    "shortcut": "available"
                }
            }
        })
    );
    assert_eq!(
        connection_payload,
        json!({
            "type": "device_connection_changed",
            "connected": true,
            "device_name": "Ableton Push 3"
        })
    );
    assert!(
        rx.recv_timeout(Duration::from_millis(100)).is_err(),
        "discovery should emit only the canonical state and device events"
    );
}

#[test]
fn system_profiler_json_extracts_nested_push_three_devices() {
    let payload = r#"{
      "SPUSBDataType": [
        {
          "_name": "USB31Bus",
          "_items": [
            {
              "_name": "USB3.1 Hub",
              "_items": [
                {
                  "_name": "Ableton Push 3",
                  "serial_num": "P3-123",
                  "location_id": "0x02110000 / 3",
                  "product_id": "0xbeef"
                }
              ]
            }
          ]
        }
      ]
    }"#;

    let devices =
        SystemDiscoverySource::from_system_profiler_json(payload).expect("payload should parse");

    assert_eq!(
        devices,
        vec![DeviceEndpointDescriptor::push_3("P3-123", "Ableton Push 3")]
    );
}

#[test]
fn system_profiler_json_dedupes_duplicate_push_three_entries() {
    let payload = r#"{
      "SPUSBDataType": [
        {
          "_name": "USB31Bus",
          "_items": [
            { "_name": "Ableton Push 3", "serial_num": "P3-123" },
            { "_name": "Ableton Push 3", "serial_num": "P3-123" }
          ]
        }
      ]
    }"#;

    let devices =
        SystemDiscoverySource::from_system_profiler_json(payload).expect("payload should parse");

    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].endpoint_id, "P3-123");
}

#[test]
fn invalid_system_profiler_json_returns_backend_error() {
    let error = SystemDiscoverySource::from_system_profiler_json("{ nope")
        .expect_err("invalid json should fail");

    #[allow(irrefutable_let_patterns)]
    let DeviceDiscoveryError::Backend(error) = error;
    assert!(
        error
            .message
            .starts_with("failed to parse system_profiler usb json:"),
        "unexpected parse error message: {}",
        error.message
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
