import { Combobox } from "@base-ui/react/combobox";
import type { CSSProperties } from "react";
import type { AppPickerOption } from "../../lib/types";

const COMMON_APP_OPTIONS = [
  {
    bundleId: "com.apple.finder",
    appName: "Finder",
  },
  {
    bundleId: "com.apple.Safari",
    appName: "Safari",
  },
  {
    bundleId: "com.google.Chrome",
    appName: "Google Chrome",
  },
  {
    bundleId: "com.apple.Terminal",
    appName: "Terminal",
  },
] as const satisfies readonly AppPickerOption[];

const pickerStyles = {
  root: {
    position: "relative",
  },
  inputGroup: {
    alignItems: "center",
    background: "rgba(245, 240, 232, 0.05)",
    border: "1px solid rgba(175, 193, 178, 0.14)",
    borderRadius: "0.95rem",
    display: "flex",
    gap: "0.45rem",
    overflow: "hidden",
    width: "100%",
  },
  input: {
    background: "transparent",
    border: "none",
    color: "#f4f0e8",
    padding: "0.8rem 0.95rem",
    flex: 1,
    minWidth: 0,
    outline: "none",
  },
  controls: {
    display: "flex",
    gap: "0.4rem",
    paddingRight: "0.55rem",
  },
  controlButton: {
    alignItems: "center",
    background: "rgba(245, 240, 232, 0.04)",
    border: "1px solid rgba(175, 193, 178, 0.12)",
    borderRadius: "999px",
    color: "#cfd8cc",
    cursor: "pointer",
    display: "inline-flex",
    height: "1.9rem",
    justifyContent: "center",
    padding: 0,
    width: "1.9rem",
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
  itemLayout: {
    alignItems: "center",
    display: "grid",
    gap: "0.25rem",
    gridTemplateColumns: "1fr auto",
  },
  itemText: {
    display: "grid",
    gap: "0.15rem",
  },
  appName: {
    color: "#f4f0e8",
    fontSize: "0.95rem",
    lineHeight: 1.2,
  },
  bundleId: {
    color: "#92a296",
    fontSize: "0.72rem",
    lineHeight: 1.2,
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

export interface AppPickerProps {
  options: readonly AppPickerOption[];
  selectedApp: AppPickerOption | null;
  disabled?: boolean;
  onSelectApp: (app: AppPickerOption | null) => void;
}

export function getEffectiveAppPickerOptions(
  options: readonly AppPickerOption[],
  selectedApp: AppPickerOption | null,
): readonly AppPickerOption[] {
  return options.length > 0
    ? options
    : selectedApp === null ||
        COMMON_APP_OPTIONS.some(
          (option) => option.bundleId === selectedApp.bundleId,
        )
      ? COMMON_APP_OPTIONS
      : [...COMMON_APP_OPTIONS, selectedApp];
}

export function AppPicker({
  options,
  selectedApp,
  disabled = false,
  onSelectApp,
}: AppPickerProps) {
  const effectiveOptions = getEffectiveAppPickerOptions(options, selectedApp);

  return (
    <Combobox.Root
      items={effectiveOptions}
      value={selectedApp}
      disabled={disabled}
      autoHighlight
      itemToStringLabel={(item: AppPickerOption) => item.appName}
      itemToStringValue={(item: AppPickerOption) => item.bundleId}
      isItemEqualToValue={(item: AppPickerOption, value: AppPickerOption) =>
        item.bundleId === value.bundleId
      }
      onValueChange={(nextApp) => {
        onSelectApp(nextApp);
      }}
    >
      <div style={pickerStyles.root}>
        <Combobox.InputGroup style={pickerStyles.inputGroup}>
          <Combobox.Input
            aria-label="App picker"
            placeholder="Search apps"
            style={pickerStyles.input}
          />
          <div style={pickerStyles.controls}>
            <Combobox.Clear
              aria-label="Clear app selection"
              style={pickerStyles.controlButton}
            >
              <ClearIcon />
            </Combobox.Clear>
            <Combobox.Trigger
              aria-label="Open app list"
              style={pickerStyles.controlButton}
            >
              <ChevronDownIcon />
            </Combobox.Trigger>
          </div>
        </Combobox.InputGroup>
        <Combobox.Portal>
          <Combobox.Positioner sideOffset={8} style={pickerStyles.positioner}>
            <Combobox.Popup style={pickerStyles.popup}>
              <Combobox.Empty style={pickerStyles.empty}>
                No matching apps.
              </Combobox.Empty>
              <Combobox.List style={pickerStyles.list}>
                {(option: AppPickerOption) => (
                  <Combobox.Item
                    key={option.bundleId}
                    value={option}
                    style={getItemStyle}
                  >
                    <div style={pickerStyles.itemLayout}>
                      <div style={pickerStyles.itemText}>
                        <span style={pickerStyles.appName}>{option.appName}</span>
                        <span style={pickerStyles.bundleId}>{option.bundleId}</span>
                      </div>
                      <Combobox.ItemIndicator style={pickerStyles.itemIndicator}>
                        Use
                      </Combobox.ItemIndicator>
                    </div>
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

function ClearIcon() {
  return (
    <svg
      aria-hidden="true"
      width="12"
      height="12"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M18 6 6 18" />
      <path d="m6 6 12 12" />
    </svg>
  );
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
