import { afterEach, describe, expect, it, vi } from "vitest";
import {
  DEFAULT_PUSH3_COLOR_CALIBRATION,
  type RuntimeEvent,
} from "./types";

const mocks = vi.hoisted(() => ({
  invokeMock: vi.fn(),
  listenMock: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invokeMock,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: mocks.listenMock,
}));

import {
  RUNTIME_EVENT_NAME,
  loadRunningApps,
  loadCurrentConfig,
  previewPush3Palette,
  refreshRuntimeState,
  restoreDefaultConfig,
  syncPush3Leds,
  subscribeRuntimeEvent,
  triggerTestAction,
  updatePush3ColorCalibration,
  updatePadBinding,
} from "./api";

afterEach(() => {
  vi.resetAllMocks();
});

describe("frontend api helpers", () => {
  it("forwards typed command wrappers to invoke with the expected shapes", async () => {
    const readyResponse = {
      status: "ready",
      config: {
        schemaVersion: 1,
        settings: {
          activeProfileId: "default",
          push3ColorCalibration: DEFAULT_PUSH3_COLOR_CALIBRATION,
        },
        profiles: [],
      },
      device_name: "Ableton Push 3",
      device_connected: true,
      runtime_state: {
        app_state: "ready",
        capabilities: {
          shortcut: "available",
        },
      },
    } as const;
    const updateResponse = {
      config: readyResponse.config,
      runtime_state: readyResponse.runtime_state,
    } as const;
    const runningAppsResponse = [
      {
        bundleId: "com.apple.Terminal",
        appName: "Terminal",
      },
    ] as const;
    const testActionResponse = {
      runtime_state: readyResponse.runtime_state,
    } as const;
    const calibrationResponse = updateResponse;
    const restoreResponse = updateResponse;

    mocks.invokeMock
      .mockResolvedValueOnce(readyResponse)
      .mockResolvedValueOnce(readyResponse)
      .mockResolvedValueOnce(runningAppsResponse)
      .mockResolvedValueOnce(updateResponse)
      .mockResolvedValueOnce(testActionResponse)
      .mockResolvedValueOnce(calibrationResponse)
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce(restoreResponse);

    await expect(loadCurrentConfig()).resolves.toBe(readyResponse);
    await expect(refreshRuntimeState()).resolves.toBe(readyResponse);
    await expect(loadRunningApps()).resolves.toBe(runningAppsResponse);
    await expect(
        updatePadBinding({
          pad_id: "r0c0",
          binding: {
          padId: "r0c0",
          label: "Launch",
          color: "green",
          action: {
            type: "launch_or_focus_app",
            bundleId: "com.apple.Terminal",
            appName: "Terminal",
          },
        },
      }),
    ).resolves.toBe(updateResponse);
    await expect(triggerTestAction("r0c0")).resolves.toBe(testActionResponse);
    await expect(
      updatePush3ColorCalibration({
        logical_color: "red",
        output_value: 9,
      }),
    ).resolves.toBe(calibrationResponse);
    await expect(previewPush3Palette({ page: 1 })).resolves.toBeUndefined();
    await expect(syncPush3Leds()).resolves.toBeUndefined();
    await expect(restoreDefaultConfig()).resolves.toBe(restoreResponse);

    expect(mocks.invokeMock).toHaveBeenNthCalledWith(1, "load_current_config");
    expect(mocks.invokeMock).toHaveBeenNthCalledWith(2, "refresh_runtime_state");
    expect(mocks.invokeMock).toHaveBeenNthCalledWith(3, "load_running_apps");
    expect(mocks.invokeMock).toHaveBeenNthCalledWith(4, "update_pad_binding", {
      request: {
        pad_id: "r0c0",
        binding: {
          padId: "r0c0",
          label: "Launch",
          color: "green",
          action: {
            type: "launch_or_focus_app",
            bundleId: "com.apple.Terminal",
            appName: "Terminal",
          },
        },
      },
    });
    expect(mocks.invokeMock).toHaveBeenNthCalledWith(5, "trigger_test_action", {
      pad_id: "r0c0",
    });
    expect(mocks.invokeMock).toHaveBeenNthCalledWith(
      6,
      "update_push3_color_calibration",
      {
        request: {
          logical_color: "red",
          output_value: 9,
        },
      },
    );
    expect(mocks.invokeMock).toHaveBeenNthCalledWith(7, "preview_push3_palette", {
      request: {
        page: 1,
      },
    });
    expect(mocks.invokeMock).toHaveBeenNthCalledWith(8, "sync_push3_leds");
    expect(mocks.invokeMock).toHaveBeenNthCalledWith(9, "restore_default_config");
  });

  it("subscribes to runtime events and returns a cleanup function", async () => {
    const runtimeEvent: RuntimeEvent = {
      type: "pad_pressed",
      pad_id: "r0c0",
    };
    const unlisten = vi.fn();
    let capturedListener: ((event: { payload: RuntimeEvent }) => void) | undefined;

    mocks.listenMock.mockImplementation(async (_eventName, listener) => {
      capturedListener = listener;
      return unlisten;
    });

    const seen: RuntimeEvent[] = [];
    const cleanup = await subscribeRuntimeEvent((event) => {
      seen.push(event);
    });

    expect(mocks.listenMock).toHaveBeenCalledWith(
      RUNTIME_EVENT_NAME,
      expect.any(Function),
    );

    capturedListener?.({ payload: runtimeEvent });
    expect(seen).toEqual([runtimeEvent]);

    await cleanup();
    expect(unlisten).toHaveBeenCalledTimes(1);
  });

  it("preserves recovery payloads from loadCurrentConfig", async () => {
    const recoveryResponse = {
      status: "recovery_required",
      device_name: null,
      device_connected: false,
      recovery: {
        config_path: "/Users/yasudanaoki/Library/Application Support/push-deck/config.json",
        backup_path:
          "/Users/yasudanaoki/Library/Application Support/push-deck/config.broken-1700000000000.json",
        reason: "invalid json",
      },
      runtime_state: {
        app_state: "config_recovery_required",
        capabilities: {
          shortcut: "unavailable",
        },
      },
    } as const;

    mocks.invokeMock.mockResolvedValueOnce(recoveryResponse);

    await expect(loadCurrentConfig()).resolves.toBe(recoveryResponse);
  });
});
