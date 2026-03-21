import type { CSSProperties } from "react";
import type {
  Config,
  ConfigRecoveryState,
  PadBinding,
  RuntimeState,
} from "../../lib/types";
import { DetailPanel } from "./DetailPanel";
import { GridView } from "./GridView";
import { RecoveryPanel } from "./RecoveryPanel";
import { StatusBar } from "../status/StatusBar";

const shellStyles = {
  frame: {
    display: "grid",
    gap: "1.5rem",
  },
  workspace: {
    display: "grid",
    gap: "1.5rem",
    gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 22rem), 1fr))",
  },
} satisfies Record<string, CSSProperties>;

export interface EditorPageProps {
  config: Config | null;
  runtimeState: RuntimeState;
  recovery: ConfigRecoveryState | null;
  selectedPadId: string | null;
  deviceName: string | null;
  isDeviceConnected: boolean;
  onRestoreDefaultConfig: () => void;
  onSelectPad: (padId: string) => void;
}

function getActiveProfile(config: Config | null) {
  if (config === null) {
    return null;
  }

  return (
    config.profiles.find(
      (profile) => profile.id === config.settings.activeProfileId,
    ) ?? config.profiles[0] ?? null
  );
}

function getSelectedPad(
  profile: { pads: PadBinding[] } | null,
  selectedPadId: string | null,
) {
  if (profile === null) {
    return null;
  }

  const fallback = profile.pads[0] ?? null;
  if (selectedPadId === null) {
    return fallback;
  }

  return profile.pads.find((pad) => pad.padId === selectedPadId) ?? fallback;
}

export function EditorPage({
  config,
  runtimeState,
  recovery,
  selectedPadId,
  deviceName,
  isDeviceConnected,
  onRestoreDefaultConfig,
  onSelectPad,
}: EditorPageProps) {
  const profile = getActiveProfile(config);
  const selectedPad = getSelectedPad(profile, selectedPadId);
  const effectiveSelectedPadId = selectedPad?.padId ?? null;
  const isRecoveryMode = runtimeState.app_state === "config_recovery_required";

  return (
    <section style={shellStyles.frame}>
      <StatusBar
        runtimeState={runtimeState}
        deviceName={deviceName}
        isDeviceConnected={isDeviceConnected}
      />
      {isRecoveryMode ? (
        <RecoveryPanel recovery={recovery} onRestoreDefaultConfig={onRestoreDefaultConfig} />
      ) : (
        <div style={shellStyles.workspace}>
          <GridView
            pads={profile?.pads ?? []}
            selectedPadId={effectiveSelectedPadId}
            onSelectPad={onSelectPad}
          />
          <DetailPanel
            pad={selectedPad}
            shortcutCapability={runtimeState.capabilities.shortcut}
          />
        </div>
      )}
    </section>
  );
}
