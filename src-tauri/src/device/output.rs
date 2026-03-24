use crate::config::schema::{Config, PadBinding, DEFAULT_PROFILE_ID};
use crate::device::colors::map_palette_value_rgb;
use crate::device::input::is_push3_user_port_display_name;
use crate::device::push3::{
    coordinate_for_pad_id, transport_pad_index_for_coordinate, Push3PadCoordinate,
};
use crate::device::push3::Push3TransportLedCommand;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[cfg(target_os = "macos")]
use coremidi::{Client, Destination, Destinations, OutputPort, PacketBuffer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Push3LedError {
    message: String,
}

impl Push3LedError {
    fn backend(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for Push3LedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for Push3LedError {}

pub trait Push3LedBackend {
    fn sync_config(&self, config: &Config) -> Result<(), Push3LedError>;
    fn preview_palette(&self, page: u8) -> Result<(), Push3LedError>;
    fn disconnect(&self);
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopPush3LedBackend;

impl Push3LedBackend for NoopPush3LedBackend {
    fn sync_config(&self, _config: &Config) -> Result<(), Push3LedError> {
        Ok(())
    }

    fn preview_palette(&self, _page: u8) -> Result<(), Push3LedError> {
        Ok(())
    }

    fn disconnect(&self) {}
}

pub fn render_config_pad_led_commands(config: &Config) -> Vec<Push3TransportLedCommand> {
    let pads = config
        .profile(&config.settings.active_profile_id)
        .or_else(|| config.profile(DEFAULT_PROFILE_ID))
        .map(|profile| profile.pads.as_slice())
        .unwrap_or(&[]);

    render_pad_binding_led_commands_with_calibration(pads, &config.settings.push3_color_calibration)
}

pub fn render_pad_binding_led_commands(pads: &[PadBinding]) -> Vec<Push3TransportLedCommand> {
    render_pad_binding_led_commands_with_calibration(
        pads,
        &crate::config::schema::Push3ColorCalibration::default(),
    )
}

pub fn render_pad_binding_led_commands_with_calibration(
    pads: &[PadBinding],
    calibration: &crate::config::schema::Push3ColorCalibration,
) -> Vec<Push3TransportLedCommand> {
    pads
        .iter()
        .filter_map(|pad| {
            let coordinate = coordinate_for_pad_id(&pad.pad_id)?;
            let transport_index = transport_pad_index_for_coordinate(coordinate)?;
            Some(Push3TransportLedCommand {
                transport_index,
                color_value: calibration.resolve(pad.color),
            })
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Push3PadRgbCommand {
    pub pad_index: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub fn render_config_pad_rgb_commands(config: &Config) -> Vec<Push3PadRgbCommand> {
    let pads = config
        .profile(&config.settings.active_profile_id)
        .or_else(|| config.profile(DEFAULT_PROFILE_ID))
        .map(|profile| profile.pads.as_slice())
        .unwrap_or(&[]);

    render_pad_binding_rgb_commands_with_calibration(pads, &config.settings.push3_color_calibration)
}

pub fn render_pad_binding_rgb_commands(pads: &[PadBinding]) -> Vec<Push3PadRgbCommand> {
    render_pad_binding_rgb_commands_with_calibration(
        pads,
        &crate::config::schema::Push3ColorCalibration::default(),
    )
}

pub fn render_pad_binding_rgb_commands_with_calibration(
    pads: &[PadBinding],
    calibration: &crate::config::schema::Push3ColorCalibration,
) -> Vec<Push3PadRgbCommand> {
    pads.iter()
        .filter_map(|pad| {
            let coordinate = coordinate_for_pad_id(&pad.pad_id)?;
            let color = map_palette_value_rgb(calibration.resolve(pad.color));
            Some(Push3PadRgbCommand {
                pad_index: (7 - coordinate.row) * 8 + coordinate.column,
                red: color.red,
                green: color.green,
                blue: color.blue,
            })
        })
        .collect()
}

pub fn encode_pad_rgb_sysex(command: Push3PadRgbCommand) -> Vec<u8> {
    vec![
        0xF0,
        0x47,
        0x7F,
        0x15,
        0x04,
        0x00,
        0x08,
        command.pad_index,
        0x00,
        command.red >> 4,
        command.red & 0x0F,
        command.green >> 4,
        command.green & 0x0F,
        command.blue >> 4,
        command.blue & 0x0F,
        0xF7,
    ]
}

pub fn encode_led_command_word(command: Push3TransportLedCommand) -> u32 {
    (0x2u32 << 28)
        | (0x90u32 << 16)
        | ((command.transport_index.0 as u32) << 8)
        | command.color_value as u32
}

pub fn encode_led_command_bytes(command: Push3TransportLedCommand) -> [u8; 3] {
    [0x90, command.transport_index.0, command.color_value]
}

pub fn render_palette_preview_page(page: u8) -> Vec<Push3TransportLedCommand> {
    let start = page.saturating_mul(64);

    (0..64u8)
        .filter_map(|offset| {
            let row = offset / 8;
            let column = offset % 8;
            let coordinate = Push3PadCoordinate { row, column };
            let transport_index = transport_pad_index_for_coordinate(coordinate)?;
            Some(Push3TransportLedCommand {
                transport_index,
                color_value: start.saturating_add(offset),
            })
        })
        .collect()
}

#[cfg(target_os = "macos")]
#[derive(Debug)]
struct Push3LedOutputConnection {
    _client: Client,
    output_port: OutputPort,
    destination: Destination,
}

#[derive(Debug)]
pub struct SystemPush3LedBackend {
    #[cfg(target_os = "macos")]
    connection: std::sync::Mutex<Option<Push3LedOutputConnection>>,
}

impl Default for SystemPush3LedBackend {
    fn default() -> Self {
        Self {
            #[cfg(target_os = "macos")]
            connection: std::sync::Mutex::new(None),
        }
    }
}

#[cfg(target_os = "macos")]
impl SystemPush3LedBackend {
    fn connect() -> Result<Option<Push3LedOutputConnection>, Push3LedError> {
        let destination = match Destinations.into_iter().find(|destination| {
            destination
                .display_name()
                .or_else(|| destination.name())
                .is_some_and(|display_name| is_push3_user_port_display_name(&display_name))
        }) {
            Some(destination) => destination,
            None => return Ok(None),
        };

        let client = Client::new("push-deck-output")
            .map_err(|status| Push3LedError::backend(format!("failed to create CoreMIDI client: {status}")))?;
        let output_port = client
            .output_port("push-deck-led-output")
            .map_err(|status| {
                Push3LedError::backend(format!("failed to create CoreMIDI output port: {status}"))
            })?;

        Ok(Some(Push3LedOutputConnection {
            _client: client,
            output_port,
            destination,
        }))
    }

    fn send_palette_commands(
        connection: &mut Push3LedOutputConnection,
        commands: &[Push3TransportLedCommand],
    ) -> Result<(), Push3LedError> {
        for command in commands {
            let message = encode_led_command_bytes(*command);
            let packet = PacketBuffer::new(0, &message);
            connection
                .output_port
                .send(&connection.destination, &packet)
                .map_err(|status| {
                    Push3LedError::backend(format!(
                        "failed to send Push 3 LED palette frame: {status}"
                    ))
                })?;
        }

        Ok(())
    }
}

#[cfg(target_os = "macos")]
impl Push3LedBackend for SystemPush3LedBackend {
    fn sync_config(&self, config: &Config) -> Result<(), Push3LedError> {
        let palette_commands = render_config_pad_led_commands(config);
        let mut connection = self.connection.lock().expect("led output lock poisoned");

        if connection.is_none() {
            *connection = Self::connect()?;
        }

        let Some(active_connection) = connection.as_mut() else {
            return Ok(());
        };

        if let Err(error) = Self::send_palette_commands(active_connection, &palette_commands) {
            *connection = None;
            return Err(error);
        }

        Ok(())
    }

    fn preview_palette(&self, page: u8) -> Result<(), Push3LedError> {
        let commands = render_palette_preview_page(page);
        let mut connection = self.connection.lock().expect("led output lock poisoned");

        if connection.is_none() {
            *connection = Self::connect()?;
        }

        let Some(active_connection) = connection.as_mut() else {
            return Ok(());
        };

        if let Err(error) = Self::send_palette_commands(active_connection, &commands) {
            *connection = None;
            return Err(error);
        }

        Ok(())
    }

    fn disconnect(&self) {
        let mut connection = self.connection.lock().expect("led output lock poisoned");
        *connection = None;
    }
}

#[cfg(not(target_os = "macos"))]
impl Push3LedBackend for SystemPush3LedBackend {
    fn sync_config(&self, _config: &Config) -> Result<(), Push3LedError> {
        Ok(())
    }

    fn preview_palette(&self, _page: u8) -> Result<(), Push3LedError> {
        Ok(())
    }

    fn disconnect(&self) {}
}
