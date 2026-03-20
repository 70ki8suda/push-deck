export type AppState =
  | "starting"
  | "waiting_for_device"
  | "ready"
  | "config_recovery_required"
  | "save_failed";

export type ShortcutCapabilityState = "available" | "unavailable";

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
