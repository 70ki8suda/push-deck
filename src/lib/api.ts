import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppPickerOption,
  CurrentConfigResponse,
  RestoreDefaultConfigResponse,
  RuntimeEvent,
  TestActionResponse,
  PreviewPush3PaletteRequest,
  UpdatePadBindingRequest,
  UpdatePadBindingResponse,
  UpdatePush3ColorCalibrationRequest,
  UpdatePush3ColorCalibrationResponse,
} from "./types";

export const RUNTIME_EVENT_NAME = "push-deck:runtime-event";

async function invokeCommand<TResult>(
  command: string,
): Promise<TResult>;
async function invokeCommand<TResult, TArgs extends Record<string, unknown>>(
  command: string,
  args: TArgs,
): Promise<TResult>;
async function invokeCommand<TResult>(
  command: string,
  args?: Record<string, unknown>,
): Promise<TResult> {
  if (args === undefined) {
    return invoke<TResult>(command);
  }

  return invoke<TResult>(command, args);
}

export function loadCurrentConfig(): Promise<CurrentConfigResponse> {
  return invokeCommand<CurrentConfigResponse>("load_current_config");
}

export function refreshRuntimeState(): Promise<CurrentConfigResponse> {
  return invokeCommand<CurrentConfigResponse>("refresh_runtime_state");
}

export function loadRunningApps(): Promise<AppPickerOption[]> {
  return invokeCommand<AppPickerOption[]>("load_running_apps");
}

export function updatePadBinding(
  request: UpdatePadBindingRequest,
): Promise<UpdatePadBindingResponse> {
  return invokeCommand<UpdatePadBindingResponse, { request: UpdatePadBindingRequest }>(
    "update_pad_binding",
    { request },
  );
}

export function triggerTestAction(
  pad_id: string,
): Promise<TestActionResponse> {
  return invokeCommand<TestActionResponse, { pad_id: string }>(
    "trigger_test_action",
    { pad_id },
  );
}

export function updatePush3ColorCalibration(
  request: UpdatePush3ColorCalibrationRequest,
): Promise<UpdatePush3ColorCalibrationResponse> {
  return invokeCommand<
    UpdatePush3ColorCalibrationResponse,
    { request: UpdatePush3ColorCalibrationRequest }
  >("update_push3_color_calibration", { request });
}

export function previewPush3Palette(
  request: PreviewPush3PaletteRequest,
): Promise<void> {
  return invokeCommand<void, { request: PreviewPush3PaletteRequest }>(
    "preview_push3_palette",
    { request },
  );
}

export function syncPush3Leds(): Promise<void> {
  return invokeCommand<void>("sync_push3_leds");
}

export function restoreDefaultConfig(): Promise<RestoreDefaultConfigResponse> {
  return invokeCommand<RestoreDefaultConfigResponse>("restore_default_config");
}

export function subscribeRuntimeEvent(
  handler: (event: RuntimeEvent) => void,
): Promise<UnlistenFn> {
  return listen<RuntimeEvent>(RUNTIME_EVENT_NAME, (event) => {
    handler(event.payload);
  });
}
