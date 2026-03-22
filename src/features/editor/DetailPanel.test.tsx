import { describe, expect, it, vi } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { DetailPanel, buildPadBindingFromDraft, clearPadBinding, createDetailDraft } from "./DetailPanel";
import { persistPadBindingEdit } from "./EditorPage";
import type {
  AppPickerOption,
  Config,
  DetailPadDraft,
  PadBinding,
  RuntimeState,
  UpdatePadBindingResponse,
} from "../../lib/types";
import { GridView } from "./GridView";

function createPadBinding(
  padId: string,
  overrides?: Partial<PadBinding>,
): PadBinding {
  return {
    padId,
    label: "",
    color: "off",
    action: {
      type: "unassigned",
    },
    ...overrides,
  };
}

function createConfig(targetPad: PadBinding): Config {
  const pads = Array.from({ length: 64 }, (_, index) => {
    const row = Math.floor(index / 8);
    const column = index % 8;
    const padId = `r${row}c${column}`;
    return padId === targetPad.padId ? targetPad : createPadBinding(padId);
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

function buildDraft(
  overrides?: Partial<DetailPadDraft>,
): DetailPadDraft {
  return {
    padId: "r0c0",
    label: "",
    color: "green",
    actionType: "unassigned",
    selectedApp: null,
    shortcutKeyInput: "",
    shortcutModifiers: [],
    ...overrides,
  };
}

describe("Task 12 detail panel editing", () => {
  it("clears a binding back to the unassigned default", () => {
    expect(
      clearPadBinding(
        createPadBinding("r0c0", {
          label: "Finder",
          color: "green",
          action: {
            type: "launch_or_focus_app",
            bundleId: "com.apple.finder",
            appName: "Finder",
          },
        }),
      ),
    ).toEqual({
      padId: "r0c0",
      label: "",
      color: "off",
      action: {
        type: "unassigned",
      },
    });
  });

  it("builds a launch binding from an app picker selection", () => {
    const terminal: AppPickerOption = {
      bundleId: "com.apple.Terminal",
      appName: "Terminal",
    };

    expect(
      buildPadBindingFromDraft(
        buildDraft({
          actionType: "launch_or_focus_app",
          selectedApp: terminal,
        }),
      ),
    ).toEqual({
      ok: true,
      binding: {
        padId: "r0c0",
        label: "Terminal",
        color: "green",
        action: {
          type: "launch_or_focus_app",
          bundleId: "com.apple.Terminal",
          appName: "Terminal",
        },
      },
    });
  });

  it("normalizes shortcut modifiers before save", () => {
    expect(
      buildPadBindingFromDraft(
        buildDraft({
          actionType: "send_shortcut",
          shortcutKeyInput: "p",
          shortcutModifiers: ["Ctrl", "Cmd", "Shift", "Cmd"],
        }),
      ),
    ).toEqual({
      ok: true,
      binding: {
        padId: "r0c0",
        label: "Cmd+Shift+Ctrl+P",
        color: "green",
        action: {
          type: "send_shortcut",
          key: "P",
          modifiers: ["Cmd", "Shift", "Ctrl"],
        },
      },
    });
  });

  it("blocks invalid shortcut input", () => {
    expect(
      buildPadBindingFromDraft(
        buildDraft({
          actionType: "send_shortcut",
          shortcutKeyInput: "?",
          shortcutModifiers: ["Cmd"],
        }),
      ),
    ).toEqual({
      ok: false,
      error: "Shortcut key must match the supported key list.",
    });
  });

  it("disables the test action button for unassigned pads", () => {
    const html = renderToStaticMarkup(
      <DetailPanel
        pad={createPadBinding("r0c0")}
        shortcutCapability="available"
      />,
    );

    expect(html).toContain("Test action");
    expect(html).toContain("disabled");
  });

  it("persists edits through the command layer and leaves the selected pad reflected in the grid", async () => {
    const updatedBinding: PadBinding = {
      padId: "r0c0",
      label: "Terminal",
      color: "blue",
      action: {
        type: "launch_or_focus_app",
        bundleId: "com.apple.Terminal",
        appName: "Terminal",
      },
    };
    const response: UpdatePadBindingResponse = {
      config: createConfig(updatedBinding),
      runtime_state: createRuntimeState(),
    };
    const updatePadBinding = vi.fn().mockResolvedValue(response);

    const result = await persistPadBindingEdit({
      binding: updatedBinding,
      updatePadBinding,
    });

    expect(updatePadBinding).toHaveBeenCalledWith({
      pad_id: "r0c0",
      binding: updatedBinding,
    });
    expect(result.selectedPadId).toBe("r0c0");

    const html = renderToStaticMarkup(
      <GridView
        pads={result.config.profiles[0].pads}
        selectedPadId={result.selectedPadId}
        onSelectPad={() => {}}
      />,
    );

    expect(html).toContain("Terminal");
    expect(html).toContain('data-pad-id="r0c0"');
    expect(html).toContain('data-selected="true"');
    expect(html).toContain("#537db9");
  });

  it("creates a draft from the selected pad for editing controls", () => {
    expect(
      createDetailDraft(
        createPadBinding("r0c1", {
          label: "Palette",
          color: "purple",
          action: {
            type: "send_shortcut",
            key: "P",
            modifiers: ["Cmd", "Shift"],
          },
        }),
      ),
    ).toEqual({
      padId: "r0c1",
      label: "Palette",
      color: "purple",
      actionType: "send_shortcut",
      selectedApp: null,
      shortcutKeyInput: "P",
      shortcutModifiers: ["Cmd", "Shift"],
    });
  });
});
