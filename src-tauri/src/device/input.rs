use crate::device::push3::{
    decode_transport_pad_input, DecodedPadInputMessage, Push3TransportPadIndex,
    Push3TransportPadInputMessage,
};
use crate::events::{emit_runtime_event, RuntimeEvent};
use tauri::{Emitter, Manager, Runtime};

#[cfg(target_os = "macos")]
use coremidi::{Client, EventList, InputPortWithContext, Protocol, Source, Sources};

#[cfg(target_os = "macos")]
type SystemInputPort<R> = InputPortWithContext<Push3InputContext<R>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Push3InputSourceDescriptor {
    pub unique_id: u32,
    pub display_name: String,
}

#[derive(Debug)]
pub enum Push3InputSubscription<R: Runtime> {
    Active(Push3InputConnection<R>),
    NotConnected,
}

#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct Push3InputConnection<R: Runtime> {
    _client: Client,
    _input_port: SystemInputPort<R>,
    _source: Source,
}

#[cfg(not(target_os = "macos"))]
#[derive(Debug)]
pub struct Push3InputConnection<R: Runtime>(std::marker::PhantomData<R>);

#[cfg(target_os = "macos")]
#[derive(Clone, Debug)]
struct Push3InputContext<R: Runtime> {
    app_handle: tauri::AppHandle<R>,
}

pub fn select_push3_user_port_source(
    sources: &[Push3InputSourceDescriptor],
) -> Option<Push3InputSourceDescriptor> {
    sources
        .iter()
        .find(|source| is_push3_user_port_display_name(&source.display_name))
        .cloned()
}

pub fn is_push3_user_port_display_name(display_name: &str) -> bool {
    let normalized = display_name.to_ascii_lowercase();
    normalized.contains("ableton push 3") && normalized.contains("user port")
}

pub fn decode_midi1_channel_voice_word(word: u32) -> Option<DecodedPadInputMessage> {
    let message_type = ((word >> 28) & 0x0f) as u8;
    if message_type != 0x2 {
        return None;
    }

    let status = ((word >> 16) & 0xff) as u8;
    let note = ((word >> 8) & 0xff) as u8;
    let velocity = (word & 0xff) as u8;
    let transport_index = Push3TransportPadIndex(note);

    match status & 0xf0 {
        0x90 if velocity > 0 => decode_transport_pad_input(
            Push3TransportPadInputMessage::PadPressed {
                transport_index,
                velocity,
            },
        ),
        0x90 | 0x80 => decode_transport_pad_input(Push3TransportPadInputMessage::PadReleased {
            transport_index,
        }),
        _ => None,
    }
}

pub fn emit_decoded_pad_input_event<R, E>(
    emitter: &E,
    message: DecodedPadInputMessage,
) -> tauri::Result<()>
where
    R: Runtime,
    E: Emitter<R>,
{
    match message {
        DecodedPadInputMessage::PadPressed { pad_id, .. } => {
            emit_runtime_event(emitter, RuntimeEvent::PadPressed { pad_id })
        }
        DecodedPadInputMessage::PadReleased { pad_id } => {
            emit_runtime_event(emitter, RuntimeEvent::PadReleased { pad_id })
        }
    }
}

#[cfg(target_os = "macos")]
pub fn subscribe_push3_user_port_runtime_events<R: Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<Push3InputSubscription<R>, String> {
    let source = match Sources
        .into_iter()
        .find(|source| {
            source
                .display_name()
                .or_else(|| source.name())
                .is_some_and(|display_name| is_push3_user_port_display_name(&display_name))
        }) {
        Some(source) => source,
        None => return Ok(Push3InputSubscription::NotConnected),
    };
    let client = Client::new("push-deck-input")
        .map_err(|status| format!("failed to create CoreMIDI client: {status}"))?;
    let mut input_port = client
        .input_port_with_protocol(
            "push-deck-user-port",
            Protocol::Midi10,
            move |event_list: &EventList, context: &mut Push3InputContext<R>| {
                for packet in event_list {
                    for word in packet.data() {
                        if let Some(message) = decode_midi1_channel_voice_word(*word) {
                            let host = context
                                .app_handle
                                .state::<crate::commands::DefaultCommandHost>();
                            let _ = crate::handle_runtime_pad_input_message(
                                &context.app_handle,
                                &host,
                                message,
                            );
                        }
                    }
                }
            },
        )
        .map_err(|status| format!("failed to create CoreMIDI input port: {status}"))?;

    input_port
        .connect_source(
            &source,
            Push3InputContext {
                app_handle: app.clone(),
            },
        )
        .map_err(|status| format!("failed to connect CoreMIDI source: {status}"))?;

    Ok(Push3InputSubscription::Active(Push3InputConnection {
        _client: client,
        _input_port: input_port,
        _source: source,
    }))
}

#[cfg(not(target_os = "macos"))]
pub fn subscribe_push3_user_port_runtime_events<R: Runtime>(
    _app: &tauri::AppHandle<R>,
) -> Result<Push3InputSubscription<R>, String> {
    Ok(Push3InputSubscription::NotConnected)
}
