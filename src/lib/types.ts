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

export const PAD_COLOR_OPTIONS = [
  "off",
  "white",
  "red",
  "orange",
  "yellow",
  "green",
  "cyan",
  "blue",
  "purple",
  "pink",
] as const satisfies readonly PadColorId[];

export const SHORTCUT_MODIFIER_ORDER = [
  "Cmd",
  "Shift",
  "Opt",
  "Ctrl",
] as const;

export type ShortcutModifier = (typeof SHORTCUT_MODIFIER_ORDER)[number];

export const SHORTCUT_KEY_OPTIONS = [
  "A",
  "B",
  "C",
  "D",
  "E",
  "F",
  "G",
  "H",
  "I",
  "J",
  "K",
  "L",
  "M",
  "N",
  "O",
  "P",
  "Q",
  "R",
  "S",
  "T",
  "U",
  "V",
  "W",
  "X",
  "Y",
  "Z",
  "0",
  "1",
  "2",
  "3",
  "4",
  "5",
  "6",
  "7",
  "8",
  "9",
  "F1",
  "F2",
  "F3",
  "F4",
  "F5",
  "F6",
  "F7",
  "F8",
  "F9",
  "F10",
  "F11",
  "F12",
  "ArrowUp",
  "ArrowDown",
  "ArrowLeft",
  "ArrowRight",
  "Space",
  "Tab",
  "Enter",
  "Escape",
  "Delete",
] as const;

export type ShortcutKey = (typeof SHORTCUT_KEY_OPTIONS)[number];

export interface AppSettings {
  activeProfileId: string;
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

export interface AppPickerOption {
  bundleId: string;
  appName: string;
}

export interface PadBinding {
  padId: string;
  label: string;
  color: PadColorId;
  action: PadAction;
}

export interface LayoutProfile {
  id: string;
  name: string;
  pads: PadBinding[];
}

export interface DetailPadDraft {
  padId: string;
  label: string;
  color: PadColorId;
  actionType: PadAction["type"];
  selectedApp: AppPickerOption | null;
  shortcutKeyInput: string;
  shortcutModifiers: ShortcutModifier[];
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
  device_name: string | null;
  device_connected: boolean;
  runtime_state: RuntimeState;
}

export interface CurrentConfigRecoveryResponse {
  status: "recovery_required";
  recovery: ConfigRecoveryState;
  device_name: string | null;
  device_connected: boolean;
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
