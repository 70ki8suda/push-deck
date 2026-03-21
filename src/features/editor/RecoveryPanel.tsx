import type { CSSProperties } from "react";
import type { ConfigRecoveryState } from "../../lib/types";

const recoveryStyles = {
  panel: {
    background:
      "linear-gradient(160deg, rgba(63, 31, 24, 0.92) 0%, rgba(29, 16, 13, 0.94) 100%)",
    border: "1px solid rgba(217, 128, 72, 0.28)",
    borderRadius: "1.75rem",
    boxShadow: "0 28px 70px rgba(12, 6, 5, 0.42)",
    color: "#f8eee8",
    display: "grid",
    gap: "1rem",
    padding: "1.5rem",
  },
  eyebrow: {
    color: "#f0b89a",
    fontSize: "0.78rem",
    letterSpacing: "0.14em",
    margin: 0,
    textTransform: "uppercase",
  },
  title: {
    fontSize: "2rem",
    margin: 0,
  },
  copy: {
    color: "#efdbcf",
    lineHeight: 1.55,
    margin: 0,
    maxWidth: "56rem",
  },
  metaList: {
    background: "rgba(255, 244, 236, 0.05)",
    border: "1px solid rgba(240, 184, 154, 0.18)",
    borderRadius: "1.25rem",
    display: "grid",
    gap: "0.85rem",
    margin: 0,
    padding: "1.1rem 1.2rem",
  },
  metaRow: {
    display: "grid",
    gap: "0.25rem",
  },
  label: {
    color: "#e5a986",
    fontSize: "0.72rem",
    letterSpacing: "0.12em",
    margin: 0,
    textTransform: "uppercase",
  },
  value: {
    color: "#fff3eb",
    margin: 0,
    overflowWrap: "anywhere",
  },
  button: {
    background: "linear-gradient(145deg, #f0dd89 0%, #d59642 100%)",
    border: "none",
    borderRadius: "999px",
    color: "#27180e",
    cursor: "pointer",
    fontSize: "0.98rem",
    fontWeight: 800,
    justifySelf: "start",
    padding: "0.95rem 1.2rem",
  },
} satisfies Record<string, CSSProperties>;

export interface RecoveryPanelProps {
  recovery: ConfigRecoveryState | null;
  onRestoreDefaultConfig: () => void;
}

export function RecoveryPanel({
  recovery,
  onRestoreDefaultConfig,
}: RecoveryPanelProps) {
  return (
    <section aria-label="Recovery panel" style={recoveryStyles.panel}>
      <p style={recoveryStyles.eyebrow}>Recovery</p>
      <h2 style={recoveryStyles.title}>Restore default layout</h2>
      <p style={recoveryStyles.copy}>
        Push Deck found a broken config file and locked the editor to avoid
        overwriting the backup. Restore the default 8x8 layout to return to the
        normal editor.
      </p>

      {recovery ? (
        <dl style={recoveryStyles.metaList}>
          <div style={recoveryStyles.metaRow}>
            <dt style={recoveryStyles.label}>Reason</dt>
            <dd style={recoveryStyles.value}>{recovery.reason}</dd>
          </div>
          <div style={recoveryStyles.metaRow}>
            <dt style={recoveryStyles.label}>Broken backup</dt>
            <dd style={recoveryStyles.value}>{recovery.backup_path}</dd>
          </div>
          <div style={recoveryStyles.metaRow}>
            <dt style={recoveryStyles.label}>Config path</dt>
            <dd style={recoveryStyles.value}>{recovery.config_path}</dd>
          </div>
        </dl>
      ) : null}

      <button
        type="button"
        onClick={onRestoreDefaultConfig}
        style={recoveryStyles.button}
      >
        Restore default layout
      </button>
    </section>
  );
}
