import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type {
  CurrentConfigResponse,
  RestoreDefaultConfigResponse,
  RuntimeEvent,
  TestActionResponse,
  UpdatePadBindingRequest,
  UpdatePadBindingResponse,
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
