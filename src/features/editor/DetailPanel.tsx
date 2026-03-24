import type { CSSProperties } from "react";
import { useEffect, useState } from "react";
import { AppPicker } from "./AppPicker";
import { ColorPicker } from "./ColorPicker";
import { padColors } from "./padPalette";
import { ShortcutEditor } from "./ShortcutEditor";
import type {
  AppPickerOption,
  DetailPadDraft,
  PadBinding,
  ShortcutCapabilityState,
  ShortcutKey,
  ShortcutModifier,
} from "../../lib/types";
import { SHORTCUT_KEY_OPTIONS, SHORTCUT_MODIFIER_ORDER } from "../../lib/types";

const detailStyles = {
  panel: {
    background: "rgba(20, 23, 21, 0.82)",
    border: "1px solid rgba(175, 193, 178, 0.18)",
    borderRadius: "1.5rem",
    boxShadow: "0 24px 60px rgba(7, 9, 8, 0.35)",
    display: "grid",
    gap: "1.25rem",
    padding: "1.35rem",
  },
  section: {
    display: "grid",
    gap: "0.45rem",
  },
  title: {
    color: "#f4f0e8",
    fontSize: "1.35rem",
    margin: 0,
  },
  meta: {
    color: "#92a296",
    fontSize: "0.82rem",
    letterSpacing: "0.08em",
    margin: 0,
    textTransform: "uppercase",
  },
  fieldLabel: {
    color: "#92a296",
    fontSize: "0.78rem",
    letterSpacing: "0.08em",
    margin: 0,
    textTransform: "uppercase",
  },
  input: {
    background: "rgba(245, 240, 232, 0.05)",
    border: "1px solid rgba(175, 193, 178, 0.14)",
    borderRadius: "0.95rem",
    color: "#f4f0e8",
    padding: "0.8rem 0.95rem",
    width: "100%",
  },
  capabilityNote: {
    background: "rgba(210, 123, 62, 0.12)",
    border: "1px solid rgba(210, 123, 62, 0.32)",
    borderRadius: "1rem",
    color: "#ffd8b4",
    margin: 0,
    padding: "0.85rem 1rem",
  },
  error: {
    background: "rgba(155, 63, 45, 0.18)",
    border: "1px solid rgba(204, 95, 71, 0.38)",
    borderRadius: "1rem",
    color: "#ffd5cb",
    margin: 0,
    padding: "0.85rem 1rem",
  },
  actions: {
    display: "grid",
    gap: "0.75rem",
    gridTemplateColumns: "repeat(3, minmax(0, 1fr))",
  },
} satisfies Record<string, CSSProperties>;

type BuildResult =
  | { ok: true; binding: PadBinding }
  | { ok: false; error: string };

function normalizeShortcutModifiers(
  modifiers: ShortcutModifier[],
): ShortcutModifier[] {
  return [...new Set(modifiers)].sort(
    (left, right) =>
      SHORTCUT_MODIFIER_ORDER.indexOf(left) -
      SHORTCUT_MODIFIER_ORDER.indexOf(right),
  );
}

export function clearPadBinding(pad: PadBinding): PadBinding {
  return {
    padId: pad.padId,
    label: "",
    color: "off",
    action: {
      type: "unassigned",
    },
  };
}

export function createDetailDraft(pad: PadBinding | null): DetailPadDraft {
  if (pad === null) {
    return {
      padId: "",
      label: "",
      color: "off",
      actionType: "unassigned",
      selectedApp: null,
      shortcutKeyInput: "",
      shortcutModifiers: [],
    };
  }

  switch (pad.action.type) {
    case "launch_or_focus_app":
      return {
        padId: pad.padId,
        label: pad.label,
        color: pad.color,
        actionType: "launch_or_focus_app",
        selectedApp: {
          bundleId: pad.action.bundleId,
          appName: pad.action.appName,
        },
        shortcutKeyInput: "",
        shortcutModifiers: [],
      };
    case "send_shortcut":
      return {
        padId: pad.padId,
        label: pad.label,
        color: pad.color,
        actionType: "send_shortcut",
        selectedApp: null,
        shortcutKeyInput: pad.action.key,
        shortcutModifiers: pad.action.modifiers,
      };
    default:
      return {
        padId: pad.padId,
        label: pad.label,
        color: pad.color,
        actionType: "unassigned",
        selectedApp: null,
        shortcutKeyInput: "",
        shortcutModifiers: [],
      };
  }
}

export function buildPadBindingFromDraft(draft: DetailPadDraft): BuildResult {
  if (draft.actionType === "launch_or_focus_app") {
    if (draft.selectedApp === null) {
      return {
        ok: false,
        error: "Choose an app before saving this pad.",
      };
    }

    return {
      ok: true,
      binding: {
        padId: draft.padId,
        label: draft.label || draft.selectedApp.appName,
        color: draft.color,
        action: {
          type: "launch_or_focus_app",
          bundleId: draft.selectedApp.bundleId,
          appName: draft.selectedApp.appName,
        },
      },
    };
  }

  if (draft.actionType === "send_shortcut") {
    const normalizedKey = draft.shortcutKeyInput.trim().toUpperCase();
    const isSupportedKey = (SHORTCUT_KEY_OPTIONS as readonly string[]).includes(
      normalizedKey,
    );

    if (!isSupportedKey) {
      return {
        ok: false,
        error: "Shortcut key must match the supported key list.",
      };
    }

    const modifiers = normalizeShortcutModifiers(draft.shortcutModifiers);
    const shortcutLabel = [modifiers.join("+"), normalizedKey]
      .filter(Boolean)
      .join("+");

    return {
      ok: true,
      binding: {
        padId: draft.padId,
        label: draft.label || shortcutLabel,
        color: draft.color,
        action: {
          type: "send_shortcut",
          key: normalizedKey as ShortcutKey,
          modifiers,
        },
      },
    };
  }

  return {
    ok: true,
    binding: {
      padId: draft.padId,
      label: draft.label,
      color: draft.color,
      action: {
        type: "unassigned",
      },
    },
  };
}

export interface DetailPanelProps {
  pad: PadBinding | null;
  appOptions?: readonly AppPickerOption[];
  shortcutCapability: ShortcutCapabilityState;
  feedbackMessage?: string | null;
  onSavePad?: (draft: DetailPadDraft) => Promise<void> | void;
  onClearPad?: (pad: PadBinding) => Promise<void> | void;
  onTestAction?: (padId: string) => Promise<void> | void;
}

export function DetailPanel({
  pad,
  appOptions = [],
  shortcutCapability,
  feedbackMessage = null,
  onSavePad,
  onClearPad,
  onTestAction,
}: DetailPanelProps) {
  const [draft, setDraft] = useState<DetailPadDraft>(() => createDetailDraft(pad));
  const [validationMessage, setValidationMessage] = useState<string | null>(null);

  useEffect(() => {
    setDraft(createDetailDraft(pad));
    setValidationMessage(null);
  }, [pad?.padId]);

  const isShortcutAction = draft.actionType === "send_shortcut";
  const isShortcutDisabled =
    isShortcutAction && shortcutCapability === "unavailable";
  const isTestDisabled =
    pad === null || draft.actionType === "unassigned" || isShortcutDisabled;
  const isSaveDisabled = pad === null;
  const headerTitle =
    draft.label ||
    (draft.actionType === "launch_or_focus_app"
      ? draft.selectedApp?.appName
      : draft.actionType === "send_shortcut"
        ? "Shortcut"
        : pad === null
          ? "Select a pad"
          : "Unassigned pad");
  const headerMeta =
    draft.actionType === "launch_or_focus_app"
      ? "Launch app"
      : draft.actionType === "send_shortcut"
        ? "Send shortcut"
        : pad === null
          ? "Grid editor"
          : "No action";

  function updateDraft(next: Partial<DetailPadDraft>) {
    setDraft((current) => ({
      ...current,
      ...next,
    }));
  }

  function validateDraft(nextDraft: DetailPadDraft) {
    const result = buildPadBindingFromDraft(nextDraft);
    setValidationMessage(result.ok ? null : result.error);
  }

  async function handleSave() {
    const result = buildPadBindingFromDraft(draft);
    if (!result.ok) {
      setValidationMessage(result.error);
      return;
    }

    setValidationMessage(null);
    await onSavePad?.(draft);
  }

  return (
    <aside aria-label="Pad details" style={detailStyles.panel}>
      <header style={detailStyles.section}>
        <h2 style={detailStyles.title}>{headerTitle}</h2>
        <p style={detailStyles.meta}>{headerMeta}</p>
      </header>

      <section style={detailStyles.section}>
        <p style={detailStyles.fieldLabel}>Label</p>
        <input
          aria-label="Pad label"
          disabled={pad === null}
          style={detailStyles.input}
          value={draft.label}
          onChange={(event) => {
            const nextDraft = { ...draft, label: event.currentTarget.value };
            setDraft(nextDraft);
            validateDraft(nextDraft);
          }}
        />
      </section>

      <section style={detailStyles.section}>
        <p style={detailStyles.fieldLabel}>Color</p>
        <ColorPicker
          disabled={pad === null}
          selectedColor={draft.color}
          onSelectColor={(color) => {
            updateDraft({ color });
          }}
        />
      </section>

      <section style={detailStyles.section}>
        <p style={detailStyles.fieldLabel}>Action type</p>
        <select
          aria-label="Action type"
          disabled={pad === null}
          style={detailStyles.input}
          value={draft.actionType}
          onChange={(event) => {
            const actionType = event.currentTarget.value as DetailPadDraft["actionType"];
            const nextDraft = {
              ...draft,
              actionType,
              selectedApp: actionType === "launch_or_focus_app" ? draft.selectedApp : null,
              shortcutKeyInput: actionType === "send_shortcut" ? draft.shortcutKeyInput : "",
              shortcutModifiers: actionType === "send_shortcut" ? draft.shortcutModifiers : [],
            };
            setDraft(nextDraft);
            validateDraft(nextDraft);
          }}
        >
          <option value="unassigned">Unassigned</option>
          <option value="launch_or_focus_app">Launch / Focus app</option>
          <option value="send_shortcut">Send shortcut</option>
        </select>
      </section>

      {draft.actionType === "launch_or_focus_app" ? (
        <section style={detailStyles.section}>
          <p style={detailStyles.fieldLabel}>Target app</p>
          <AppPicker
            options={appOptions}
            selectedApp={draft.selectedApp}
            disabled={pad === null}
            onSelectApp={(selectedApp: AppPickerOption | null) => {
              const nextDraft = {
                ...draft,
                selectedApp,
                label: draft.label || selectedApp?.appName || "",
              };
              setDraft(nextDraft);
              validateDraft(nextDraft);
            }}
          />
        </section>
      ) : null}

      {draft.actionType === "send_shortcut" ? (
        <section style={detailStyles.section}>
          <p style={detailStyles.fieldLabel}>Shortcut</p>
          <ShortcutEditor
            keyInput={draft.shortcutKeyInput}
            modifiers={draft.shortcutModifiers}
            disabled={pad === null}
            validationMessage={validationMessage}
            onKeyInputChange={(shortcutKeyInput) => {
              const nextDraft = { ...draft, shortcutKeyInput };
              setDraft(nextDraft);
              validateDraft(nextDraft);
            }}
            onModifiersChange={(shortcutModifiers) => {
              const nextDraft = { ...draft, shortcutModifiers };
              setDraft(nextDraft);
              validateDraft(nextDraft);
            }}
          />
        </section>
      ) : null}

      {isShortcutDisabled ? (
        <p style={detailStyles.capabilityNote}>
          Shortcut execution unavailable until Accessibility permission is granted.
        </p>
      ) : null}

      {validationMessage && draft.actionType !== "send_shortcut" ? (
        <p style={detailStyles.error}>{validationMessage}</p>
      ) : null}

      {feedbackMessage ? <p style={detailStyles.meta}>{feedbackMessage}</p> : null}

      <div style={detailStyles.actions}>
        <button
          type="button"
          disabled={pad === null}
          style={getActionButtonStyle(pad !== null)}
          onClick={() => {
            if (pad) {
              void onClearPad?.(pad);
            }
          }}
        >
          Clear binding
        </button>
        <button
          type="button"
          disabled={isSaveDisabled}
          style={getActionButtonStyle(!isSaveDisabled)}
          onClick={() => {
            void handleSave();
          }}
        >
          Save binding
        </button>
        <button
          type="button"
          disabled={isTestDisabled}
          style={getActionButtonStyle(!isTestDisabled)}
          onClick={() => {
            if (pad !== null) {
              void onTestAction?.(pad.padId);
            }
          }}
        >
          Test action
        </button>
      </div>
    </aside>
  );
}

function getActionButtonStyle(isEnabled: boolean): CSSProperties {
  return {
    background: isEnabled
      ? "linear-gradient(145deg, #f0dd89 0%, #cc9d3a 100%)"
      : "rgba(245, 240, 232, 0.08)",
    border: "1px solid rgba(175, 193, 178, 0.14)",
    borderRadius: "999px",
    color: isEnabled ? "#1f190d" : "#819183",
    cursor: isEnabled ? "pointer" : "not-allowed",
    fontWeight: 700,
    padding: "0.85rem 1rem",
  };
}
