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
