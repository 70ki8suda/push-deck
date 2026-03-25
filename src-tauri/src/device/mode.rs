use tauri::{Manager, Runtime};

#[cfg(target_os = "macos")]
use coremidi::{Client, InputPort, PacketList, Source, Sources};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushModeEvent {
    UserModeButtonPressed,
    UserModeButtonReleased,
    UserModeEntered,
    UserModeExited,
}

const USER_MODE_CONTROLLER: u8 = 0x3B;
const PUSH_MODE_ENTERED_SYSEX: [u8; 9] = [0xF0, 0x00, 0x21, 0x1D, 0x01, 0x01, 0x0A, 0x01, 0xF7];
const PUSH_MODE_EXITED_SYSEX: [u8; 9] = [0xF0, 0x00, 0x21, 0x1D, 0x01, 0x01, 0x0A, 0x00, 0xF7];

pub fn decode_push_mode_message(bytes: &[u8]) -> Option<PushModeEvent> {
    match bytes {
        [0xB0, controller, 0x7F] if *controller == USER_MODE_CONTROLLER => {
            Some(PushModeEvent::UserModeButtonPressed)
        }
        [0xB0, controller, 0x00] if *controller == USER_MODE_CONTROLLER => {
            Some(PushModeEvent::UserModeButtonReleased)
        }
        _ if bytes == PUSH_MODE_ENTERED_SYSEX => Some(PushModeEvent::UserModeEntered),
        _ if bytes == PUSH_MODE_EXITED_SYSEX => Some(PushModeEvent::UserModeExited),
        _ => None,
    }
}

#[derive(Debug)]
pub enum Push3ModeSubscription {
    Active(Push3ModeConnection),
    NotConnected,
}

#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct Push3ModeConnection {
    _client: Client,
    _input_port: InputPort,
    _sources: Vec<Source>,
}

#[cfg(not(target_os = "macos"))]
#[derive(Debug)]
pub struct Push3ModeConnection;

pub fn is_push3_mode_port_display_name(display_name: &str) -> bool {
    let normalized = display_name.to_ascii_lowercase();
    normalized.contains("ableton push 3")
        && (normalized.contains("live port") || normalized.contains("user port"))
}

#[cfg(target_os = "macos")]
pub fn subscribe_push3_mode_runtime_events<R: Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<Push3ModeSubscription, String> {
    let sources: Vec<Source> = Sources
        .into_iter()
        .filter(|source| {
            source
                .display_name()
                .or_else(|| source.name())
                .is_some_and(|display_name| is_push3_mode_port_display_name(&display_name))
        })
        .collect();

    if sources.is_empty() {
        return Ok(Push3ModeSubscription::NotConnected);
    }

    let client = Client::new("push-deck-mode")
        .map_err(|status| format!("failed to create CoreMIDI mode client: {status}"))?;
    let app_handle = app.clone();
    let input_port = client
        .input_port("push-deck-mode-port", move |packet_list: &PacketList| {
            for packet in packet_list {
                if let Some(event) = decode_push_mode_message(packet.data()) {
                    let host = app_handle.state::<crate::commands::DefaultCommandHost>();
                    let _ = crate::handle_push_mode_event(&app_handle, &host, event);
                }
            }
        })
        .map_err(|status| format!("failed to create CoreMIDI mode input port: {status}"))?;

    for source in &sources {
        input_port
            .connect_source(source)
            .map_err(|status| format!("failed to connect CoreMIDI mode source: {status}"))?;
    }

    Ok(Push3ModeSubscription::Active(Push3ModeConnection {
        _client: client,
        _input_port: input_port,
        _sources: sources,
    }))
}

#[cfg(not(target_os = "macos"))]
pub fn subscribe_push3_mode_runtime_events<R: Runtime>(
    _app: &tauri::AppHandle<R>,
) -> Result<Push3ModeSubscription, String> {
    Ok(Push3ModeSubscription::NotConnected)
}
