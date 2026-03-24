import type { CSSProperties } from "react";
import type { PadBinding } from "../../lib/types";
import { padColors } from "./padPalette";

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
  onMovePad?: (sourcePadId: string, targetPadId: string) => void;
}

export function GridView({
  pads,
  selectedPadId,
  onSelectPad,
  onMovePad,
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
              draggable
              onClick={() => {
                onSelectPad(pad.padId);
              }}
              onDragStart={(event) => {
                event.dataTransfer.effectAllowed = "move";
                event.dataTransfer.setData("text/pad-id", pad.padId);
              }}
              onDragOver={(event) => {
                event.preventDefault();
                event.dataTransfer.dropEffect = "move";
              }}
              onDrop={(event) => {
                event.preventDefault();
                const sourcePadId = event.dataTransfer.getData("text/pad-id");
                if (sourcePadId && sourcePadId !== pad.padId) {
                  onMovePad?.(sourcePadId, pad.padId);
                }
              }}
              style={{
                aspectRatio: "1.28 / 1",
                background: palette.background,
                border: "1px solid rgba(255, 255, 255, 0.08)",
                borderRadius: "1rem",
                boxShadow: isSelected
                  ? "inset 0 0 0 2px #f3d26b, 0 0 0 2px rgba(243, 210, 107, 0.16), 0 14px 24px rgba(7, 9, 8, 0.22)"
                  : "0 12px 20px rgba(7, 9, 8, 0.16)",
                boxSizing: "border-box",
                color: palette.foreground,
                cursor: "pointer",
                display: "flex",
                flexDirection: "column",
                justifyContent: "center",
                alignItems: "flex-start",
                overflow: "hidden",
                minWidth: 0,
                padding: "0.65rem 0.75rem",
                textAlign: "left",
                transition: "transform 140ms ease, box-shadow 140ms ease",
              }}
            >
              <span
                style={{
                  display: "-webkit-box",
                  fontSize: "0.82rem",
                  fontWeight: 600,
                  lineHeight: 1.15,
                  minWidth: 0,
                  overflow: "hidden",
                  overflowWrap: "anywhere",
                  WebkitBoxOrient: "vertical",
                  WebkitLineClamp: 2,
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
