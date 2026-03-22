import type { CSSProperties } from "react";
import type { ShortcutModifier } from "../../lib/types";
import { SHORTCUT_MODIFIER_ORDER } from "../../lib/types";

const shortcutStyles = {
  stack: {
    display: "grid",
    gap: "0.75rem",
  },
  modifierRow: {
    display: "flex",
    flexWrap: "wrap",
    gap: "0.5rem",
  },
  modifierButton: {
    border: "1px solid rgba(175, 193, 178, 0.14)",
    borderRadius: "999px",
    color: "#f4f0e8",
    cursor: "pointer",
    padding: "0.55rem 0.8rem",
  },
  keyInput: {
    background: "rgba(245, 240, 232, 0.05)",
    border: "1px solid rgba(175, 193, 178, 0.14)",
    borderRadius: "0.95rem",
    color: "#f4f0e8",
    padding: "0.8rem 0.95rem",
  },
  hint: {
    color: "#92a296",
    fontSize: "0.9rem",
    margin: 0,
  },
  error: {
    color: "#ffd5cb",
    fontSize: "0.9rem",
    margin: 0,
  },
} satisfies Record<string, CSSProperties>;

export interface ShortcutEditorProps {
  keyInput: string;
  modifiers: ShortcutModifier[];
  disabled?: boolean;
  validationMessage?: string | null;
  onKeyInputChange: (value: string) => void;
  onModifiersChange: (value: ShortcutModifier[]) => void;
}

export function ShortcutEditor({
  keyInput,
  modifiers,
  disabled = false,
  validationMessage = null,
  onKeyInputChange,
  onModifiersChange,
}: ShortcutEditorProps) {
  return (
    <div style={shortcutStyles.stack}>
      <div style={shortcutStyles.modifierRow}>
        {SHORTCUT_MODIFIER_ORDER.map((modifier) => {
          const isActive = modifiers.includes(modifier);

          return (
            <button
              key={modifier}
              type="button"
              disabled={disabled}
              style={{
                ...shortcutStyles.modifierButton,
                background: isActive
                  ? "linear-gradient(145deg, #5a9b57 0%, #2f6236 100%)"
                  : "rgba(245, 240, 232, 0.05)",
                opacity: disabled ? 0.65 : 1,
              }}
              onClick={() => {
                const nextModifiers = isActive
                  ? modifiers.filter((value) => value !== modifier)
                  : [...modifiers, modifier];
                onModifiersChange(nextModifiers);
              }}
            >
              {modifier}
            </button>
          );
        })}
      </div>

      <input
        aria-label="Shortcut key"
        disabled={disabled}
        placeholder="P, F1, ArrowUp, Space"
        style={shortcutStyles.keyInput}
        value={keyInput}
        onChange={(event) => {
          onKeyInputChange(event.currentTarget.value);
        }}
      />

      <p style={validationMessage ? shortcutStyles.error : shortcutStyles.hint}>
        {validationMessage ??
          "Allowed keys: A-Z, 0-9, F1-F12, arrows, Space, Tab, Enter, Escape, Delete."}
      </p>
    </div>
  );
}
