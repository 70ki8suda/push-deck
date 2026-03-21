import { describe, expect, it, vi } from "vitest";
import {
  createRuntimeSubscription,
  deriveLoadedState,
  deriveRestoredState,
} from "../../App";
import type {
  Config,
  CurrentConfigResponse,
  RestoreDefaultConfigResponse,
  RuntimeEvent,
  RuntimeState,
} from "../../lib/types";

function createConfig(): Config {
  return {
    schemaVersion: 1,
    settings: {
      activeProfileId: "default",
    },
    profiles: [
      {
        id: "default",
        name: "Default",
        pads: [
          {
            padId: "r0c0",
            label: "Finder",
            color: "green",
            action: {
              type: "launch_or_focus_app",
              bundleId: "com.apple.finder",
              appName: "Finder",
            },
          },
        ],
      },
    ],
  };
}

function createRuntimeState(overrides?: Partial<RuntimeState>): RuntimeState {
  return {
    app_state: "ready",
    capabilities: {
      shortcut: "available",
      ...overrides?.capabilities,
    },
    ...overrides,
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });

  return { promise, resolve, reject };
}

describe("Task 11 app shell runtime wiring", () => {
  it("maps recovery payloads into locked-down editor state", () => {
    const response: CurrentConfigResponse = {
      status: "recovery_required",
      recovery: {
        config_path: "/tmp/config.json",
        backup_path: "/tmp/config.broken.json",
        reason: "invalid json",
      },
      runtime_state: createRuntimeState({
        app_state: "config_recovery_required",
        capabilities: {
          shortcut: "unavailable",
        },
      }),
    };

    expect(deriveLoadedState(response)).toEqual({
      config: null,
      recovery: response.recovery,
      runtimeState: response.runtime_state,
      selectedPadId: null,
    });
  });

  it("maps restore responses back to the default selected pad", () => {
    const response: RestoreDefaultConfigResponse = {
      config: createConfig(),
      runtime_state: createRuntimeState(),
    };

    expect(deriveRestoredState(response)).toEqual({
      config: response.config,
      recovery: null,
      runtimeState: response.runtime_state,
      selectedPadId: "r0c0",
    });
  });

  it("cleans up a late runtime subscription after teardown", async () => {
    const subscribeGate = deferred<() => void>();
    const unlisten = vi.fn();
    const applyLoadedState = vi.fn();
    const handleRuntimeEvent = vi.fn();
    const handleLoadError = vi.fn();
    let capturedListener: ((event: RuntimeEvent) => void) | null = null;

    const teardown = createRuntimeSubscription({
      loadCurrentConfig: vi.fn().mockResolvedValue({
        status: "ready",
        config: createConfig(),
        runtime_state: createRuntimeState(),
      } satisfies CurrentConfigResponse),
      subscribeRuntimeEvent: vi.fn(async (listener) => {
        capturedListener = listener;
        return subscribeGate.promise;
      }),
      applyLoadedState,
      handleRuntimeEvent,
      handleLoadError,
    });

    await Promise.resolve();
    await Promise.resolve();

    teardown();
    subscribeGate.resolve(unlisten);

    await Promise.resolve();
    await Promise.resolve();

    capturedListener?.({
      type: "state_changed",
      state: createRuntimeState({
        app_state: "waiting_for_device",
      }),
    });

    expect(applyLoadedState).toHaveBeenCalledTimes(1);
    expect(handleRuntimeEvent).not.toHaveBeenCalled();
    expect(handleLoadError).not.toHaveBeenCalled();
    expect(unlisten).toHaveBeenCalledTimes(1);
  });

  it("surfaces load errors through the app bootstrap handler", async () => {
    const applyLoadedState = vi.fn();
    const handleRuntimeEvent = vi.fn();
    const handleLoadError = vi.fn();

    createRuntimeSubscription({
      loadCurrentConfig: vi.fn().mockRejectedValue(new Error("boom")),
      subscribeRuntimeEvent: vi.fn(),
      applyLoadedState,
      handleRuntimeEvent,
      handleLoadError,
    });

    await Promise.resolve();
    await Promise.resolve();

    expect(applyLoadedState).not.toHaveBeenCalled();
    expect(handleRuntimeEvent).not.toHaveBeenCalled();
    expect(handleLoadError).toHaveBeenCalledTimes(1);
    expect(handleLoadError).toHaveBeenCalledWith(expect.any(Error));
  });
});
