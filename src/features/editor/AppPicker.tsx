import type { CSSProperties } from "react";
import type { AppPickerOption } from "../../lib/types";

const pickerStyles = {
  select: {
    background: "rgba(245, 240, 232, 0.05)",
    border: "1px solid rgba(175, 193, 178, 0.14)",
    borderRadius: "0.95rem",
    color: "#f4f0e8",
    padding: "0.8rem 0.95rem",
    width: "100%",
  },
} satisfies Record<string, CSSProperties>;

export interface AppPickerProps {
  options: readonly AppPickerOption[];
  selectedApp: AppPickerOption | null;
  disabled?: boolean;
  onSelectApp: (app: AppPickerOption | null) => void;
}

export function AppPicker({
  options,
  selectedApp,
  disabled = false,
  onSelectApp,
}: AppPickerProps) {
  return (
    <select
      aria-label="App picker"
      disabled={disabled}
      value={selectedApp?.bundleId ?? ""}
      style={pickerStyles.select}
      onChange={(event) => {
        const nextApp =
          options.find((option) => option.bundleId === event.currentTarget.value) ??
          null;
        onSelectApp(nextApp);
      }}
    >
      <option value="">
        {options.length > 0 ? "Choose an app" : "No installed apps loaded"}
      </option>
      {options.map((option) => (
        <option key={option.bundleId} value={option.bundleId}>
          {option.appName}
        </option>
      ))}
    </select>
  );
}
