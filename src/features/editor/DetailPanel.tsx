import type { CSSProperties } from "react";
import type { PadBinding, ShortcutCapabilityState } from "../../lib/types";

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
  eyebrow: {
    color: "#a8b7aa",
    fontSize: "0.78rem",
    letterSpacing: "0.14em",
    margin: 0,
    textTransform: "uppercase",
  },
  title: {
    color: "#f4f0e8",
    fontSize: "1.5rem",
    margin: 0,
  },
  meta: {
    color: "#c8d0c8",
    margin: 0,
  },
  fieldLabel: {
    color: "#92a296",
    fontSize: "0.78rem",
    letterSpacing: "0.08em",
    margin: 0,
    textTransform: "uppercase",
  },
  fieldValue: {
    background: "rgba(245, 240, 232, 0.05)",
    border: "1px solid rgba(175, 193, 178, 0.14)",
    borderRadius: "1rem",
    color: "#f4f0e8",
    margin: 0,
    padding: "0.9rem 1rem",
  },
  capabilityNote: {
    background: "rgba(210, 123, 62, 0.12)",
    border: "1px solid rgba(210, 123, 62, 0.32)",
    borderRadius: "1rem",
    color: "#ffd8b4",
    margin: 0,
    padding: "0.85rem 1rem",
  },
  actions: {
    display: "grid",
    gap: "0.75rem",
    gridTemplateColumns: "repeat(2, minmax(0, 1fr))",
  },
} satisfies Record<string, CSSProperties>;

function describeAction(pad: PadBinding | null) {
  if (pad === null) {
    return "Select a pad to inspect its binding.";
  }

  switch (pad.action.type) {
    case "launch_or_focus_app":
      return `Launch or focus ${pad.action.appName}`;
    case "send_shortcut":
      return `${pad.action.modifiers.join("+")}+${pad.action.key}`;
    default:
      return "No action assigned";
  }
}

export interface DetailPanelProps {
  pad: PadBinding | null;
  shortcutCapability: ShortcutCapabilityState;
}

export function DetailPanel({
  pad,
  shortcutCapability,
}: DetailPanelProps) {
  const isShortcutAction = pad?.action.type === "send_shortcut";
  const isShortcutDisabled =
    isShortcutAction && shortcutCapability === "unavailable";
  const isTestDisabled =
    pad === null || pad.action.type === "unassigned" || isShortcutDisabled;

  return (
    <aside aria-label="Pad details" style={detailStyles.panel}>
      <header style={detailStyles.section}>
        <p style={detailStyles.eyebrow}>Editor</p>
        <h2 style={detailStyles.title}>Pad details</h2>
        <p style={detailStyles.meta}>
          {pad ? `${pad.padId} is selected` : "Pick any pad from the grid to inspect it."}
        </p>
      </header>

      <section style={detailStyles.section}>
        <p style={detailStyles.fieldLabel}>Label</p>
        <p style={detailStyles.fieldValue}>{pad?.label || "Unassigned"}</p>
      </section>

      <section style={detailStyles.section}>
        <p style={detailStyles.fieldLabel}>Action</p>
        <p style={detailStyles.fieldValue}>{describeAction(pad)}</p>
      </section>

      <section style={detailStyles.section}>
        <p style={detailStyles.fieldLabel}>Color</p>
        <p style={detailStyles.fieldValue}>{pad?.color ?? "off"}</p>
      </section>

      {isShortcutDisabled ? (
        <p style={detailStyles.capabilityNote}>
          Shortcut execution unavailable until Accessibility permission is granted.
        </p>
      ) : null}

      <div style={detailStyles.actions}>
        <button
          type="button"
          disabled={pad === null}
          style={getActionButtonStyle(pad !== null)}
        >
          Clear binding
        </button>
        <button
          type="button"
          disabled={isTestDisabled}
          style={getActionButtonStyle(!isTestDisabled)}
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
