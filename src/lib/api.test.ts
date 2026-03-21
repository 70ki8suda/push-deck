import { afterEach, describe, expect, it, vi } from "vitest";
import type { RuntimeEvent } from "./types";

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
  loadCurrentConfig,
  restoreDefaultConfig,
  subscribeRuntimeEvent,
  triggerTestAction,
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
          active_profile_id: "default",
        },
        profiles: [],
      },
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
    const testActionResponse = {
      runtime_state: readyResponse.runtime_state,
    } as const;
    const restoreResponse = updateResponse;

    mocks.invokeMock
      .mockResolvedValueOnce(readyResponse)
      .mockResolvedValueOnce(updateResponse)
      .mockResolvedValueOnce(testActionResponse)
      .mockResolvedValueOnce(restoreResponse);

    await expect(loadCurrentConfig()).resolves.toBe(readyResponse);
    await expect(
      updatePadBinding({
        pad_id: "r0c0",
        binding: {
          pad_id: "r0c0",
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
    await expect(restoreDefaultConfig()).resolves.toBe(restoreResponse);

    expect(mocks.invokeMock).toHaveBeenNthCalledWith(1, "load_current_config");
    expect(mocks.invokeMock).toHaveBeenNthCalledWith(2, "update_pad_binding", {
      request: {
        pad_id: "r0c0",
        binding: {
          pad_id: "r0c0",
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
    expect(mocks.invokeMock).toHaveBeenNthCalledWith(3, "trigger_test_action", {
      pad_id: "r0c0",
    });
    expect(mocks.invokeMock).toHaveBeenNthCalledWith(4, "restore_default_config");
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
