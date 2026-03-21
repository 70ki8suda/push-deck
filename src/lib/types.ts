export type AppState =
  | "starting"
  | "waiting_for_device"
  | "ready"
  | "config_recovery_required"
  | "save_failed";

export type ShortcutCapabilityState = "available" | "unavailable";

export type PadColorId =
  | "off"
  | "white"
  | "red"
  | "orange"
  | "yellow"
  | "green"
  | "cyan"
  | "blue"
  | "purple"
  | "pink";

export type ShortcutModifier = "Cmd" | "Shift" | "Opt" | "Ctrl";

export type ShortcutKey =
  | "A"
  | "B"
  | "C"
  | "D"
  | "E"
  | "F"
  | "G"
  | "H"
  | "I"
  | "J"
  | "K"
  | "L"
  | "M"
  | "N"
  | "O"
  | "P"
  | "Q"
  | "R"
  | "S"
  | "T"
  | "U"
  | "V"
  | "W"
  | "X"
  | "Y"
  | "Z"
  | "0"
  | "1"
  | "2"
  | "3"
  | "4"
  | "5"
  | "6"
  | "7"
  | "8"
  | "9"
  | "F1"
  | "F2"
  | "F3"
  | "F4"
  | "F5"
  | "F6"
  | "F7"
  | "F8"
  | "F9"
  | "F10"
  | "F11"
  | "F12"
  | "ArrowUp"
  | "ArrowDown"
  | "ArrowLeft"
  | "ArrowRight"
  | "Space"
  | "Tab"
  | "Enter"
  | "Escape"
  | "Delete";

export interface AppSettings {
  active_profile_id: string;
}

export interface PadActionUnassigned {
  type: "unassigned";
}

export interface PadActionLaunchOrFocusApp {
  type: "launch_or_focus_app";
  bundleId: string;
  appName: string;
}

export interface PadActionSendShortcut {
  type: "send_shortcut";
  key: ShortcutKey;
  modifiers: ShortcutModifier[];
}

export type PadAction =
  | PadActionUnassigned
  | PadActionLaunchOrFocusApp
  | PadActionSendShortcut;

export interface PadBinding {
  pad_id: string;
  label: string;
  color: PadColorId;
  action: PadAction;
}

export interface LayoutProfile {
  id: string;
  name: string;
  pads: PadBinding[];
}

export interface Config {
  schemaVersion: number;
  settings: AppSettings;
  profiles: LayoutProfile[];
}

export interface ConfigRecoveryState {
  config_path: string;
  backup_path: string;
  reason: string;
}

export interface CurrentConfigReadyResponse {
  status: "ready";
  config: Config;
  runtime_state: RuntimeState;
}

export interface CurrentConfigRecoveryResponse {
  status: "recovery_required";
  recovery: ConfigRecoveryState;
  runtime_state: RuntimeState;
}

export type CurrentConfigResponse =
  | CurrentConfigReadyResponse
  | CurrentConfigRecoveryResponse;

export interface RuntimeCapabilities {
  shortcut: ShortcutCapabilityState;
}

export interface RuntimeState {
  app_state: AppState;
  capabilities: RuntimeCapabilities;
}

export type DisplayTarget = "main" | "top-strip";

export interface DisplayFrame {
  target: DisplayTarget;
  payload: unknown;
}

export interface UpdatePadBindingRequest {
  pad_id: string;
  binding: PadBinding;
}

export interface UpdatePadBindingResponse {
  config: Config;
  runtime_state: RuntimeState;
}

export interface TestActionResponse {
  runtime_state: RuntimeState;
}

export interface RestoreDefaultConfigResponse {
  config: Config;
  runtime_state: RuntimeState;
}

export type RuntimeEvent =
  | {
      type: "state_changed";
      state: RuntimeState;
    }
  | {
      type: "device_connection_changed";
      connected: boolean;
      device_name: string | null;
    }
  | {
      type: "pad_pressed";
      pad_id: string;
    }
  | {
      type: "display_frame";
      frame: DisplayFrame;
    };
