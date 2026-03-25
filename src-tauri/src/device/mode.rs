use tauri::Runtime;

#[cfg(target_os = "macos")]
use coremidi::{Client, EventList, InputPortWithContext, Protocol, Source, Sources};
#[cfg(target_os = "macos")]
use coremidi_sys::{MIDIEventList, MIDIEventPacket};

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

#[cfg(target_os = "macos")]
type SystemModeInputPort<R> = InputPortWithContext<Push3ModeContext<R>>;

#[cfg(target_os = "macos")]
#[derive(Clone, Debug)]
struct Push3ModeContext<R: Runtime> {
    app_handle: tauri::AppHandle<R>,
}

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

pub fn decode_midi1_push_mode_word(word: u32) -> Option<PushModeEvent> {
    let message_type = ((word >> 28) & 0x0f) as u8;
    if message_type != 0x2 {
        return None;
    }

    let status = ((word >> 16) & 0xff) as u8;
    let controller = ((word >> 8) & 0xff) as u8;
    let value = (word & 0xff) as u8;

    match status & 0xf0 {
        0xB0 if controller == USER_MODE_CONTROLLER && value == 0x7F => {
            Some(PushModeEvent::UserModeButtonPressed)
        }
        0xB0 if controller == USER_MODE_CONTROLLER && value == 0x00 => {
            Some(PushModeEvent::UserModeButtonReleased)
        }
        _ => None,
    }
}

#[cfg(target_os = "macos")]
pub fn decode_push_mode_event_list(event_list: &EventList) -> Option<PushModeEvent> {
    unsafe {
        let event_list_ptr = std::ptr::from_ref(event_list).cast::<MIDIEventList>();
        let packet_ptr = std::ptr::addr_of!((*event_list_ptr).packet).cast::<MIDIEventPacket>();
        let word_count_ptr = std::ptr::addr_of!((*packet_ptr).wordCount);
        let word_count = word_count_ptr.read_unaligned() as usize;
        let words_ptr = std::ptr::addr_of!((*packet_ptr).words).cast::<u32>();

        for index in 0..word_count {
            let word = words_ptr.add(index).read_unaligned();
            if let Some(event) = decode_midi1_push_mode_word(word) {
                return Some(event);
            }
        }
    }

    None
}

#[derive(Debug)]
pub enum Push3ModeSubscription<R: Runtime> {
    Active(Push3ModeConnection<R>),
    NotConnected,
}

#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct Push3ModeConnection<R: Runtime> {
    _client: Client,
    _input_port: SystemModeInputPort<R>,
    _sources: Vec<Source>,
}

#[cfg(not(target_os = "macos"))]
#[derive(Debug)]
pub struct Push3ModeConnection<R: Runtime>(std::marker::PhantomData<R>);

pub fn is_push3_mode_port_display_name(display_name: &str) -> bool {
    let normalized = display_name.to_ascii_lowercase();
    normalized.contains("ableton push 3")
        && (normalized.contains("live port") || normalized.contains("user port"))
}

#[cfg(target_os = "macos")]
pub fn subscribe_push3_mode_runtime_events<R: Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<Push3ModeSubscription<R>, String> {
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
    let mut input_port = client
        .input_port_with_protocol(
            "push-deck-mode-port",
            Protocol::Midi10,
            move |event_list: &EventList, context: &mut Push3ModeContext<R>| {
                if let Some(event) = decode_push_mode_event_list(event_list) {
                    crate::schedule_push_mode_event_handling(
                        context.app_handle.clone(),
                        event,
                    );
                }
            },
        )
        .map_err(|status| format!("failed to create CoreMIDI mode input port: {status}"))?;

    for source in &sources {
        input_port
            .connect_source(
                source,
                Push3ModeContext {
                    app_handle: app.clone(),
                },
            )
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
) -> Result<Push3ModeSubscription<R>, String> {
    Ok(Push3ModeSubscription::NotConnected)
}
