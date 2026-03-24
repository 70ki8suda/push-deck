import { Combobox } from "@base-ui/react/combobox";
import type { CSSProperties } from "react";
import { PAD_COLOR_OPTIONS } from "../../lib/types";
import type { PadColorId } from "../../lib/types";
import { padColors } from "./padPalette";

type ColorOption = {
  id: PadColorId;
  label: string;
};

const COLOR_OPTIONS = PAD_COLOR_OPTIONS.map((color) => ({
  id: color,
  label: labelForColor(color),
})) satisfies readonly ColorOption[];

const pickerStyles = {
  root: {
    position: "relative",
  },
  inputGroup: {
    alignItems: "center",
    border: "1px solid rgba(175, 193, 178, 0.14)",
    borderRadius: "0.95rem",
    display: "flex",
    gap: "0.7rem",
    overflow: "hidden",
    paddingLeft: "0.8rem",
    width: "100%",
  },
  preview: {
    borderRadius: "0.7rem",
    boxShadow: "inset 0 0 0 1px rgba(255, 255, 255, 0.18)",
    flexShrink: 0,
    height: "1.35rem",
    width: "1.35rem",
  },
  input: {
    background: "transparent",
    border: "none",
    color: "inherit",
    flex: 1,
    minWidth: 0,
    outline: "none",
    padding: "0.8rem 0",
  },
  controlButton: {
    alignItems: "center",
    background: "rgba(245, 240, 232, 0.08)",
    border: "none",
    color: "inherit",
    cursor: "pointer",
    display: "inline-flex",
    height: "100%",
    justifyContent: "center",
    minWidth: "2.8rem",
    padding: "0 0.75rem",
  },
  positioner: {
    zIndex: 10,
  },
  popup: {
    background: "rgba(20, 23, 21, 0.96)",
    border: "1px solid rgba(175, 193, 178, 0.18)",
    borderRadius: "1rem",
    boxShadow: "0 18px 42px rgba(7, 9, 8, 0.42)",
    marginTop: "0.4rem",
    maxHeight: "18rem",
    overflow: "hidden",
    width: "min(28rem, max(16rem, var(--anchor-width)))",
  },
  list: {
    display: "grid",
    gap: "0.2rem",
    maxHeight: "18rem",
    overflowY: "auto",
    padding: "0.45rem",
  },
  item: {
    alignItems: "center",
    display: "grid",
    gap: "0.75rem",
    gridTemplateColumns: "1.4rem 1fr auto",
  },
  itemLabel: {
    color: "#f4f0e8",
    fontSize: "0.95rem",
    lineHeight: 1.2,
  },
  itemMeta: {
    color: "#92a296",
    fontSize: "0.72rem",
    letterSpacing: "0.08em",
    textTransform: "uppercase",
  },
  itemIndicator: {
    color: "#f0dd89",
    fontSize: "0.76rem",
    fontWeight: 700,
    letterSpacing: "0.08em",
    textTransform: "uppercase",
  },
  empty: {
    color: "#92a296",
    padding: "0.95rem 1rem",
    textAlign: "center",
    width: "100%",
  },
} satisfies Record<string, CSSProperties>;

export interface ColorPickerProps {
  selectedColor: PadColorId;
  disabled?: boolean;
  onSelectColor: (color: PadColorId) => void;
}

export function ColorPicker({
  selectedColor,
  disabled = false,
  onSelectColor,
}: ColorPickerProps) {
  const selectedOption = COLOR_OPTIONS.find((option) => option.id === selectedColor)
    ?? COLOR_OPTIONS[0];
  const selectedPalette = padColors[selectedOption.id];

  return (
    <Combobox.Root
      items={COLOR_OPTIONS}
      value={selectedOption}
      disabled={disabled}
      autoHighlight
      itemToStringLabel={(item: ColorOption) => item.label}
      itemToStringValue={(item: ColorOption) => item.id}
      isItemEqualToValue={(item: ColorOption, value: ColorOption) => item.id === value.id}
      onValueChange={(nextOption) => {
        onSelectColor(nextOption?.id ?? "off");
      }}
    >
      <div style={pickerStyles.root}>
        <Combobox.InputGroup
          style={{
            ...pickerStyles.inputGroup,
            background: selectedPalette.background,
            color: selectedPalette.foreground,
          }}
        >
          <span
            aria-hidden="true"
            style={{
              ...pickerStyles.preview,
              background: selectedPalette.background,
            }}
          />
          <Combobox.Input
            aria-label="Pad color"
            placeholder="Search colors"
            style={pickerStyles.input}
          />
          <Combobox.Trigger
            aria-label="Open color list"
            style={pickerStyles.controlButton}
          >
            <ChevronDownIcon />
          </Combobox.Trigger>
        </Combobox.InputGroup>
        <Combobox.Portal>
          <Combobox.Positioner sideOffset={8} style={pickerStyles.positioner}>
            <Combobox.Popup style={pickerStyles.popup}>
              <Combobox.Empty style={pickerStyles.empty}>
                No matching colors.
              </Combobox.Empty>
              <Combobox.List style={pickerStyles.list}>
                {(option: ColorOption) => (
                  <Combobox.Item
                    key={option.id}
                    value={option}
                    style={getItemStyle}
                  >
                    <div style={pickerStyles.item}>
                      <span
                        aria-hidden="true"
                        style={{
                          ...pickerStyles.preview,
                          background: padColors[option.id].background,
                        }}
                      />
                      <span style={pickerStyles.itemLabel}>{option.label}</span>
                      <span style={pickerStyles.itemMeta}>App color</span>
                    </div>
                    <Combobox.ItemIndicator style={pickerStyles.itemIndicator}>
                      Use
                    </Combobox.ItemIndicator>
                  </Combobox.Item>
                )}
              </Combobox.List>
            </Combobox.Popup>
          </Combobox.Positioner>
        </Combobox.Portal>
      </div>
    </Combobox.Root>
  );
}

function getItemStyle(
  state: {
    highlighted: boolean;
    selected: boolean;
  },
): CSSProperties {
  return {
    background:
      state.highlighted || state.selected
        ? "rgba(240, 221, 137, 0.14)"
        : "transparent",
    border: "1px solid rgba(175, 193, 178, 0.08)",
    borderRadius: "0.85rem",
    cursor: "pointer",
    padding: "0.7rem 0.8rem",
  };
}

function labelForColor(color: PadColorId) {
  return color.slice(0, 1).toUpperCase() + color.slice(1);
}

function ChevronDownIcon() {
  return (
    <svg
      aria-hidden="true"
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="m6 9 6 6 6-6" />
    </svg>
  );
}
