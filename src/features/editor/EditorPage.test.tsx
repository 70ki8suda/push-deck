// @vitest-environment jsdom

import { afterEach, describe, expect, it, vi } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { EditorPage, swapPadBindings, persistPadBindingSwap } from "./EditorPage";
import { GridView } from "./GridView";
import { StatusBar } from "../status/StatusBar";
import type {
  Config,
  CurrentConfigRecoveryResponse,
  PadBinding,
  RuntimeState,
} from "../../lib/types";
import { DEFAULT_PUSH3_COLOR_CALIBRATION } from "../../lib/types";

afterEach(() => {
  cleanup();
  vi.unstubAllEnvs();
});

function createPadBinding(padId: string): PadBinding {
  return {
    padId,
    label: padId === "r0c0" ? "Finder" : "",
    color: padId === "r0c0" ? "green" : "off",
    action:
      padId === "r0c0"
        ? {
            type: "launch_or_focus_app",
            bundleId: "com.apple.finder",
            appName: "Finder",
          }
        : {
            type: "unassigned",
          },
  };
}

function createConfig(): Config {
  const pads = Array.from({ length: 64 }, (_, index) => {
    const row = Math.floor(index / 8);
    const column = index % 8;
    return createPadBinding(`r${row}c${column}`);
  });

  return {
    schemaVersion: 1,
    settings: {
      activeProfileId: "default",
      push3ColorCalibration: DEFAULT_PUSH3_COLOR_CALIBRATION,
    },
    profiles: [
      {
        id: "default",
        name: "Default",
        pads,
      },
    ],
  };
}

function createRuntimeState(
  overrides?: Partial<RuntimeState>,
): RuntimeState {
  return {
    app_state: "ready",
    capabilities: {
      shortcut: "available",
      ...overrides?.capabilities,
    },
    ...overrides,
  };
}

function createRecoveryResponse(): CurrentConfigRecoveryResponse {
  return {
    status: "recovery_required",
    device_name: null,
    device_connected: false,
    recovery: {
      config_path:
        "/Users/yasudanaoki/Library/Application Support/push-deck/config.json",
      backup_path:
        "/Users/yasudanaoki/Library/Application Support/push-deck/config.broken-1700000000.json",
      reason: "invalid json",
    },
    runtime_state: createRuntimeState({
      app_state: "config_recovery_required",
      capabilities: {
        shortcut: "unavailable",
      },
    }),
  };
}

describe("Task 11 editor shell", () => {
  it("swaps two pad bindings within the active profile", () => {
    const config = createConfig();
    config.profiles[0].pads[1] = {
      padId: "r0c1",
      label: "Terminal",
      color: "blue",
      action: {
        type: "launch_or_focus_app",
        bundleId: "com.apple.Terminal",
        appName: "Terminal",
      },
    };

    const swapped = swapPadBindings(config, "r0c0", "r0c1");

    expect(swapped.profiles[0].pads[0]).toMatchObject({
      padId: "r0c0",
      label: "Terminal",
      color: "blue",
    });
    expect(swapped.profiles[0].pads[1]).toMatchObject({
      padId: "r0c1",
      label: "Finder",
      color: "green",
    });
  });

  it("persists a pad swap by updating both pad ids", async () => {
    const config = createConfig();
    config.profiles[0].pads[1] = {
      padId: "r0c1",
      label: "Terminal",
      color: "blue",
      action: {
        type: "launch_or_focus_app",
        bundleId: "com.apple.Terminal",
        appName: "Terminal",
      },
    };
    const updatePadBinding = vi
      .fn()
      .mockResolvedValueOnce({
        config: swapPadBindings(config, "r0c0", "r0c1"),
        runtime_state: createRuntimeState(),
      })
      .mockResolvedValueOnce({
        config: swapPadBindings(config, "r0c0", "r0c1"),
        runtime_state: createRuntimeState(),
      });

    const result = await persistPadBindingSwap({
      config,
      sourcePadId: "r0c0",
      targetPadId: "r0c1",
      updatePadBinding,
    });

    expect(updatePadBinding).toHaveBeenCalledTimes(2);
    expect(updatePadBinding).toHaveBeenNthCalledWith(1, {
      pad_id: "r0c0",
      binding: expect.objectContaining({
        padId: "r0c0",
        label: "Terminal",
      }),
    });
    expect(updatePadBinding).toHaveBeenNthCalledWith(2, {
      pad_id: "r0c1",
      binding: expect.objectContaining({
        padId: "r0c1",
        label: "Finder",
      }),
    });
    expect(result.selectedPadId).toBe("r0c1");
  });

  it("renders the full 8x8 grid", () => {
    const html = renderToStaticMarkup(
      <GridView
        pads={createConfig().profiles[0].pads}
        selectedPadId="r0c0"
        onSelectPad={() => {}}
      />,
    );

    expect(html.match(/data-pad-id=\"r\dc\d\"/g)).toHaveLength(64);
    expect(html.indexOf('data-pad-id="r0c0"')).toBeLessThan(
      html.indexOf('data-pad-id="r7c7"'),
    );
  });

  it("marks the selected pad in the grid", () => {
    const html = renderToStaticMarkup(
      <GridView
        pads={createConfig().profiles[0].pads}
        selectedPadId="r3c4"
        onSelectPad={() => {}}
      />,
    );

    expect(html).toContain('data-pad-id="r3c4"');
    expect(html).toContain('data-selected="true"');
  });

  it("renders status state including color mapping controls", () => {
    const html = renderToStaticMarkup(
      <StatusBar
        runtimeState={createRuntimeState({
          app_state: "waiting_for_device",
          capabilities: {
            shortcut: "unavailable",
          },
        })}
        deviceName="Ableton Push 3"
        isDeviceConnected={false}
        canToggleColorMapping
        isColorMappingVisible={false}
      />,
    );

    expect(html).toContain("Waiting for device");
    expect(html).toContain("Device offline");
    expect(html).toContain("Ableton Push 3");
    expect(html).toContain("Color mapping");
    expect(html).toContain("Mapping hidden");
    expect(html).toContain("Show mapping");
  });

  it("surfaces shortcut-disabled state in the detail panel shell", () => {
    const config = createConfig();
    config.profiles[0].pads[1] = {
      padId: "r0c1",
      label: "Command palette",
      color: "blue",
      action: {
        type: "send_shortcut",
        key: "P",
        modifiers: ["Cmd", "Shift"],
      },
    };

    const html = renderToStaticMarkup(
      <EditorPage
        config={config}
        runtimeState={createRuntimeState({
          capabilities: {
            shortcut: "unavailable",
          },
        })}
        recovery={null}
        selectedPadId="r0c1"
        deviceName="Ableton Push 3"
        isDeviceConnected
        onRestoreDefaultConfig={() => {}}
        onSelectPad={() => {}}
      />,
    );

    expect(html).toContain("Shortcut execution unavailable");
    expect(html).toContain("Test action");
    expect(html).toContain("disabled");
  });

  it("hides the push3 calibration controls by default", () => {
    const html = renderToStaticMarkup(
      <EditorPage
        config={createConfig()}
        runtimeState={createRuntimeState()}
        recovery={null}
        selectedPadId="r0c0"
        deviceName="Ableton Push 3"
        isDeviceConnected
        onRestoreDefaultConfig={() => {}}
        onSelectPad={() => {}}
      />,
    );

    expect(html).not.toContain("Preview 0-63");
    expect(html).toContain("Show mapping");
  });

  it("keeps the push3 calibration controls hidden until toggled", () => {
    const html = renderToStaticMarkup(
      <EditorPage
        config={createConfig()}
        runtimeState={createRuntimeState()}
        recovery={null}
        selectedPadId="r0c0"
        deviceName="Ableton Push 3"
        isDeviceConnected
        onRestoreDefaultConfig={() => {}}
        onSelectPad={() => {}}
      />,
    );

    expect(html).not.toContain("Preview 0-63");
    expect(html).toContain("Show mapping");
  });

  it("shows and hides the push3 calibration controls from the status bar toggle", async () => {
    const user = userEvent.setup();

    render(
      <EditorPage
        config={createConfig()}
        runtimeState={createRuntimeState()}
        recovery={null}
        selectedPadId="r0c0"
        deviceName="Ableton Push 3"
        isDeviceConnected
        onRestoreDefaultConfig={() => {}}
        onSelectPad={() => {}}
      />,
    );

    expect(screen.queryByText("Push 3 palette match")).toBeNull();

    await user.click(screen.getByRole("button", { name: "Show mapping" }));

    expect(screen.getByText("Push 3 palette match")).toBeTruthy();
    expect(screen.getByText("Preview 0-63")).toBeTruthy();

    await user.click(screen.getByRole("button", { name: "Hide mapping" }));

    expect(screen.queryByText("Push 3 palette match")).toBeNull();
  });

  it("can disable the color mapping tools explicitly with the env flag", () => {
    vi.stubEnv("VITE_SHOW_PUSH3_CALIBRATION", "false");

    const html = renderToStaticMarkup(
      <EditorPage
        config={createConfig()}
        runtimeState={createRuntimeState()}
        recovery={null}
        selectedPadId="r0c0"
        deviceName="Ableton Push 3"
        isDeviceConnected
        onRestoreDefaultConfig={() => {}}
        onSelectPad={() => {}}
      />,
    );

    expect(html).toContain("Unavailable in this build");
    expect(html).not.toContain("Show mapping");
    expect(html).not.toContain("Preview 0-63");
  });

  it("replaces normal editor actions with the recovery flow", () => {
    const recovery = createRecoveryResponse();

    const html = renderToStaticMarkup(
      <EditorPage
        config={null}
        runtimeState={recovery.runtime_state}
        recovery={recovery.recovery}
        selectedPadId={null}
        deviceName={null}
        isDeviceConnected={false}
        onRestoreDefaultConfig={() => {}}
        onSelectPad={() => {}}
      />,
    );

    expect(html).toContain("Restore default layout");
    expect(html).toContain("config.broken-1700000000.json");
    expect(html).not.toContain("Pad details");
    expect(html).not.toContain("Clear binding");
    expect(html).not.toContain("Test action");
  });
});
