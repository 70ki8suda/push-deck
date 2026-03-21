import type { CSSProperties } from "react";
import type { PadBinding, PadColorId } from "../../lib/types";

const padColors: Record<PadColorId, { background: string; foreground: string }> = {
  off: {
    background: "linear-gradient(145deg, #2a312d 0%, #1b201d 100%)",
    foreground: "#b3beb6",
  },
  white: {
    background: "linear-gradient(145deg, #f4efe4 0%, #ddd6c7 100%)",
    foreground: "#1d211d",
  },
  red: {
    background: "linear-gradient(145deg, #d35f43 0%, #9f3d25 100%)",
    foreground: "#fff7f2",
  },
  orange: {
    background: "linear-gradient(145deg, #dd8b39 0%, #b35d1a 100%)",
    foreground: "#fff6ec",
  },
  yellow: {
    background: "linear-gradient(145deg, #dbc95d 0%, #b79d27 100%)",
    foreground: "#241f12",
  },
  green: {
    background: "linear-gradient(145deg, #5a9b57 0%, #2f6236 100%)",
    foreground: "#edf9ee",
  },
  cyan: {
    background: "linear-gradient(145deg, #57a8a4 0%, #276d73 100%)",
    foreground: "#ecfeff",
  },
  blue: {
    background: "linear-gradient(145deg, #537db9 0%, #2a4d83 100%)",
    foreground: "#eff6ff",
  },
  purple: {
    background: "linear-gradient(145deg, #7669bc 0%, #493d85 100%)",
    foreground: "#f3efff",
  },
  pink: {
    background: "linear-gradient(145deg, #cb6c96 0%, #92385d 100%)",
    foreground: "#fff0f6",
  },
};

const gridStyles = {
  panel: {
    background: "rgba(20, 23, 21, 0.82)",
    border: "1px solid rgba(175, 193, 178, 0.18)",
    borderRadius: "1.5rem",
    boxShadow: "0 24px 60px rgba(7, 9, 8, 0.35)",
    padding: "1.25rem",
  },
  header: {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "baseline",
    gap: "1rem",
    marginBottom: "1rem",
  },
  label: {
    color: "#a8b7aa",
    fontSize: "0.78rem",
    letterSpacing: "0.14em",
    margin: 0,
    textTransform: "uppercase",
  },
  title: {
    color: "#f4f0e8",
    fontSize: "1.3rem",
    margin: 0,
  },
  grid: {
    display: "grid",
    gridTemplateColumns: "repeat(8, minmax(0, 1fr))",
    gap: "0.65rem",
  },
} satisfies Record<string, CSSProperties>;

export interface GridViewProps {
  pads: PadBinding[];
  selectedPadId: string | null;
  onSelectPad: (padId: string) => void;
}

export function GridView({
  pads,
  selectedPadId,
  onSelectPad,
}: GridViewProps) {
  return (
    <section aria-label="Pad grid" style={gridStyles.panel}>
      <div style={gridStyles.header}>
        <div>
          <p style={gridStyles.label}>Layout</p>
          <h2 style={gridStyles.title}>8x8 Grid</h2>
        </div>
        <p style={gridStyles.label}>{pads.length}/64 pads</p>
      </div>
      <div style={gridStyles.grid}>
        {pads.map((pad) => {
          const palette = padColors[pad.color];
          const isSelected = pad.padId === selectedPadId;

          return (
            <button
              key={pad.padId}
              type="button"
              data-pad-id={pad.padId}
              data-selected={isSelected}
              onClick={() => {
                onSelectPad(pad.padId);
              }}
              style={{
                aspectRatio: "1 / 1",
                background: palette.background,
                border: isSelected
                  ? "2px solid #f3d26b"
                  : "1px solid rgba(255, 255, 255, 0.08)",
                borderRadius: "1rem",
                boxShadow: isSelected
                  ? "0 0 0 2px rgba(243, 210, 107, 0.16), 0 14px 24px rgba(7, 9, 8, 0.22)"
                  : "0 12px 20px rgba(7, 9, 8, 0.16)",
                color: palette.foreground,
                cursor: "pointer",
                display: "grid",
                gap: "0.2rem",
                justifyItems: "start",
                padding: "0.7rem",
                textAlign: "left",
                transition: "transform 140ms ease, box-shadow 140ms ease",
              }}
            >
              <span
                style={{
                  fontSize: "0.7rem",
                  letterSpacing: "0.08em",
                  opacity: 0.82,
                  textTransform: "uppercase",
                }}
              >
                {pad.padId}
              </span>
              <span
                style={{
                  fontSize: "0.9rem",
                  fontWeight: 600,
                  lineHeight: 1.15,
                }}
              >
                {pad.label || "Unassigned"}
              </span>
            </button>
          );
        })}
      </div>
    </section>
  );
}
