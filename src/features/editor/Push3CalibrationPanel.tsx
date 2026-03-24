import type { CSSProperties } from "react";
import { useState } from "react";
import {
  DEFAULT_PUSH3_COLOR_CALIBRATION,
  EDITABLE_PUSH3_CALIBRATION_COLORS,
} from "../../lib/types";
import type { PadColorId, Push3ColorCalibration } from "../../lib/types";
import { pushPalettePage } from "./pushPalette";
import { padColors } from "./padPalette";

const panelStyles = {
  section: {
    background: "rgba(20, 23, 21, 0.82)",
    border: "1px solid rgba(175, 193, 178, 0.18)",
    borderRadius: "1.5rem",
    boxShadow: "0 24px 60px rgba(7, 9, 8, 0.2)",
    display: "grid",
    gap: "1rem",
    padding: "1.2rem 1.35rem",
  },
  header: {
    display: "grid",
    gap: "0.25rem",
  },
  title: {
    color: "#f4f0e8",
    fontSize: "1.05rem",
    margin: 0,
  },
  note: {
    color: "#9bb09d",
    fontSize: "0.84rem",
    margin: 0,
  },
  controls: {
    display: "flex",
    flexWrap: "wrap",
    gap: "0.65rem",
  },
  controlButton: {
    background: "rgba(245, 240, 232, 0.07)",
    border: "1px solid rgba(175, 193, 178, 0.14)",
    borderRadius: "999px",
    color: "#f4f0e8",
    cursor: "pointer",
    padding: "0.6rem 0.9rem",
  },
  targetGrid: {
    display: "grid",
    gap: "0.6rem",
    gridTemplateColumns: "repeat(auto-fit, minmax(10rem, 1fr))",
  },
  targetButton: {
    alignItems: "center",
    background: "rgba(245, 240, 232, 0.05)",
    border: "1px solid rgba(175, 193, 178, 0.14)",
    borderRadius: "1rem",
    color: "#f4f0e8",
    cursor: "pointer",
    display: "grid",
    gap: "0.5rem",
    gridTemplateColumns: "1fr auto",
    padding: "0.75rem 0.85rem",
    textAlign: "left",
  },
  targetLabel: {
    margin: 0,
  },
  previewSwatch: {
    borderRadius: "999px",
    boxShadow: "inset 0 0 0 1px rgba(255, 255, 255, 0.18)",
    height: "0.7rem",
    width: "0.7rem",
  },
  value: {
    color: "#9bb09d",
    fontSize: "0.78rem",
  },
  paletteGrid: {
    display: "grid",
    gap: "0.45rem",
    gridTemplateColumns: "repeat(8, minmax(0, 1fr))",
  },
  paletteButton: {
    background: "rgba(245, 240, 232, 0.04)",
    border: "1px solid rgba(175, 193, 178, 0.14)",
    borderRadius: "0.9rem",
    color: "#d8e1d9",
    cursor: "pointer",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    minWidth: 0,
    minHeight: "3.25rem",
    padding: "0.55rem 0.45rem",
  },
  paletteLabel: {
    fontSize: "0.72rem",
    fontWeight: 700,
    letterSpacing: "0.04em",
  },
} satisfies Record<string, CSSProperties>;

type EditableCalibrationColor = Exclude<PadColorId, "off">;

export interface Push3CalibrationPanelProps {
  calibration?: Push3ColorCalibration;
  onPreviewPage?: (page: number) => Promise<void> | void;
  onRestoreLayout?: () => Promise<void> | void;
  onUpdateCalibration?: (
    logicalColor: EditableCalibrationColor,
    outputValue: number,
  ) => Promise<void> | void;
}

function labelForColor(color: string) {
  return color.slice(0, 1).toUpperCase() + color.slice(1);
}

export function Push3CalibrationPanel({
  calibration = DEFAULT_PUSH3_COLOR_CALIBRATION,
  onPreviewPage,
  onRestoreLayout,
  onUpdateCalibration,
}: Push3CalibrationPanelProps) {
  const [activeLogicalColor, setActiveLogicalColor] =
    useState<EditableCalibrationColor>("red");
  const [page, setPage] = useState(0);

  const palette = pushPalettePage(page);

  async function handlePreviewPage(nextPage: number) {
    setPage(nextPage);
    await onPreviewPage?.(nextPage);
  }

  return (
    <section style={panelStyles.section}>
      <header style={panelStyles.header}>
        <h2 style={panelStyles.title}>Push 3 palette match</h2>
        <p style={panelStyles.note}>
          Preview palette pages on Push, then click the swatch that matches the color you want.
        </p>
      </header>

      <div style={panelStyles.controls}>
        <button
          type="button"
          style={panelStyles.controlButton}
          onClick={() => {
            void handlePreviewPage(0);
          }}
        >
          Preview 0-63
        </button>
        <button
          type="button"
          style={panelStyles.controlButton}
          onClick={() => {
            void handlePreviewPage(1);
          }}
        >
          Preview 64-127
        </button>
        <button
          type="button"
          style={panelStyles.controlButton}
          onClick={() => {
            void onRestoreLayout?.();
          }}
        >
          Return to layout
        </button>
      </div>

      <div style={panelStyles.targetGrid}>
        {EDITABLE_PUSH3_CALIBRATION_COLORS.map((color) => {
          const assignedValue = calibration[color];
          const isActive = activeLogicalColor === color;

          return (
            <button
              key={color}
              type="button"
              style={{
                ...panelStyles.targetButton,
                boxShadow: isActive ? "inset 0 0 0 2px #f3d26b" : undefined,
              }}
              onClick={() => {
                setActiveLogicalColor(color);
              }}
            >
              <span
                style={{
                  alignItems: "center",
                  display: "inline-flex",
                  gap: "0.45rem",
                }}
              >
                <span
                  style={{
                    ...panelStyles.previewSwatch,
                    background: padColors[color].background,
                  }}
                />
                <span style={panelStyles.targetLabel}>App {labelForColor(color)}</span>
              </span>
              <span style={panelStyles.value}>#{assignedValue}</span>
            </button>
          );
        })}
      </div>

      <div style={panelStyles.paletteGrid}>
        {palette.map((swatch) => (
          <button
            key={swatch.value}
            type="button"
            aria-label={`Palette ${swatch.value}`}
            style={{
              ...panelStyles.paletteButton,
              boxShadow:
                calibration[activeLogicalColor] === swatch.value
                  ? "inset 0 0 0 2px #f3d26b"
                  : "inset 0 0 0 1px rgba(255, 255, 255, 0.18)",
            }}
            onClick={() => {
              void (async () => {
                await onUpdateCalibration?.(activeLogicalColor, swatch.value);
                await onPreviewPage?.(page);
              })();
            }}
          >
            <span style={panelStyles.paletteLabel}>#{swatch.value}</span>
          </button>
        ))}
      </div>
    </section>
  );
}
