import type { CSSProperties } from "react";
import type { RuntimeState } from "../../lib/types";

const statusStyles = {
  bar: {
    alignItems: "stretch",
    display: "grid",
    gap: "0.9rem",
    gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 16rem), 1fr))",
  },
  card: {
    background: "rgba(20, 23, 21, 0.82)",
    border: "1px solid rgba(175, 193, 178, 0.18)",
    borderRadius: "1.25rem",
    boxShadow: "0 20px 45px rgba(7, 9, 8, 0.24)",
    display: "grid",
    gap: "0.4rem",
    minHeight: "6rem",
    padding: "1rem 1.1rem",
  },
  label: {
    color: "#8ea18f",
    fontSize: "0.74rem",
    letterSpacing: "0.14em",
    margin: 0,
    textTransform: "uppercase",
  },
  value: {
    color: "#f4f0e8",
    fontSize: "1.15rem",
    fontWeight: 700,
    margin: 0,
  },
  detail: {
    color: "#bac6bb",
    margin: 0,
  },
} satisfies Record<string, CSSProperties>;

function formatAppState(appState: RuntimeState["app_state"]) {
  switch (appState) {
    case "waiting_for_device":
      return "Waiting for device";
    case "config_recovery_required":
      return "Recovery required";
    case "save_failed":
      return "Save failed";
    case "ready":
      return "Ready";
    default:
      return "Starting";
  }
}

export interface StatusBarProps {
  runtimeState: RuntimeState;
  deviceName: string | null;
  isDeviceConnected: boolean;
}

export function StatusBar({
  runtimeState,
  deviceName,
  isDeviceConnected,
}: StatusBarProps) {
  return (
    <section aria-label="Runtime status" style={statusStyles.bar}>
      <article style={statusStyles.card}>
        <p style={statusStyles.label}>App state</p>
        <p style={statusStyles.value}>{formatAppState(runtimeState.app_state)}</p>
        <p style={statusStyles.detail}>
          {runtimeState.app_state === "config_recovery_required"
            ? "Editor actions are locked until the default layout is restored."
            : "Runtime updates stream here as the Rust core changes state."}
        </p>
      </article>

      <article style={statusStyles.card}>
        <p style={statusStyles.label}>Device</p>
        <p style={statusStyles.value}>
          {isDeviceConnected ? "Device connected" : "Device offline"}
        </p>
        <p style={statusStyles.detail}>{deviceName ?? "No Push 3 detected yet"}</p>
      </article>

      <article style={statusStyles.card}>
        <p style={statusStyles.label}>Shortcut capability</p>
        <p style={statusStyles.value}>
          {runtimeState.capabilities.shortcut === "available"
            ? "Shortcut capability available"
            : "Shortcut capability unavailable"}
        </p>
        <p style={statusStyles.detail}>
          {runtimeState.capabilities.shortcut === "available"
            ? "Shortcut actions can be tested from the selected pad."
            : "Grant Accessibility permission to enable shortcut actions and tests."}
        </p>
      </article>
    </section>
  );
}
