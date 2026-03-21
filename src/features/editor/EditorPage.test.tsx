import { describe, expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { EditorPage } from "./EditorPage";
import { GridView } from "./GridView";
import { StatusBar } from "../status/StatusBar";
import type {
  Config,
  CurrentConfigRecoveryResponse,
  PadBinding,
  RuntimeState,
} from "../../lib/types";

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

  it("renders status state including disabled shortcut capability", () => {
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
      />,
    );

    expect(html).toContain("Waiting for device");
    expect(html).toContain("Device offline");
    expect(html).toContain("Ableton Push 3");
    expect(html).toContain("Shortcut capability unavailable");
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
