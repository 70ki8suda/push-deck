import { startTransition, useEffect, useState } from "react";
import type { CSSProperties } from "react";
import { triggerTestAction, updatePadBinding } from "../../lib/api";
import type {
  AppPickerOption,
  Config,
  ConfigRecoveryState,
  DetailPadDraft,
  PadBinding,
  RuntimeState,
  TestActionResponse,
  UpdatePadBindingResponse,
} from "../../lib/types";
import { DetailPanel, buildPadBindingFromDraft, clearPadBinding } from "./DetailPanel";
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
  appOptions?: readonly AppPickerOption[];
  deviceName: string | null;
  isDeviceConnected: boolean;
  onRestoreDefaultConfig: () => void;
  onSelectPad: (padId: string) => void;
}

export function getActiveProfile(config: Config | null) {
  if (config === null) {
    return null;
  }

  return (
    config.profiles.find(
      (profile) => profile.id === config.settings.activeProfileId,
    ) ?? config.profiles[0] ?? null
  );
}

export function getSelectedPad(
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

export async function persistPadBindingEdit({
  binding,
  updatePadBinding: updatePadBindingCommand = updatePadBinding,
}: {
  binding: PadBinding;
  updatePadBinding?: (
    request: { pad_id: string; binding: PadBinding },
  ) => Promise<UpdatePadBindingResponse>;
}) {
  const response = await updatePadBindingCommand({
    pad_id: binding.padId,
    binding,
  });

  return {
    config: response.config,
    runtimeState: response.runtime_state,
    selectedPadId: binding.padId,
  };
}

async function runTestAction(
  padId: string,
  triggerTestActionCommand = triggerTestAction,
) {
  const response: TestActionResponse = await triggerTestActionCommand(padId);
  return response.runtime_state;
}

export function EditorPage({
  config,
  runtimeState,
  recovery,
  selectedPadId,
  appOptions = [],
  deviceName,
  isDeviceConnected,
  onRestoreDefaultConfig,
  onSelectPad,
}: EditorPageProps) {
  const [localConfig, setLocalConfig] = useState(config);
  const [localRuntimeState, setLocalRuntimeState] = useState(runtimeState);
  const [feedbackMessage, setFeedbackMessage] = useState<string | null>(null);

  useEffect(() => {
    setLocalConfig(config);
  }, [config]);

  useEffect(() => {
    setLocalRuntimeState(runtimeState);
  }, [runtimeState]);

  const profile = getActiveProfile(localConfig);
  const selectedPad = getSelectedPad(profile, selectedPadId);
  const effectiveSelectedPadId = selectedPad?.padId ?? null;
  const isRecoveryMode = localRuntimeState.app_state === "config_recovery_required";

  async function applySavedBinding(binding: PadBinding) {
    const next = await persistPadBindingEdit({ binding });

    startTransition(() => {
      setLocalConfig(next.config);
      setLocalRuntimeState(next.runtimeState);
      setFeedbackMessage(`Saved ${binding.padId}.`);
      onSelectPad(next.selectedPadId);
    });
  }

  async function handleSavePad(draft: DetailPadDraft) {
    const result = buildPadBindingFromDraft(draft);
    if (!result.ok) {
      startTransition(() => {
        setFeedbackMessage(result.error);
      });
      return;
    }

    try {
      await applySavedBinding(result.binding);
    } catch (error) {
      startTransition(() => {
        setFeedbackMessage(
          error instanceof Error ? error.message : "Unable to save this pad binding.",
        );
      });
    }
  }

  async function handleClearPad(pad: PadBinding) {
    try {
      await applySavedBinding(clearPadBinding(pad));
    } catch (error) {
      startTransition(() => {
        setFeedbackMessage(
          error instanceof Error ? error.message : "Unable to clear this pad binding.",
        );
      });
    }
  }

  async function handleTestAction(padId: string) {
    try {
      const nextRuntimeState = await runTestAction(padId);
      startTransition(() => {
        setLocalRuntimeState(nextRuntimeState);
        setFeedbackMessage(`Tested ${padId}.`);
      });
    } catch (error) {
      startTransition(() => {
        setFeedbackMessage(
          error instanceof Error ? error.message : "Unable to test this pad binding.",
        );
      });
    }
  }

  return (
    <section style={shellStyles.frame}>
      <StatusBar
        runtimeState={localRuntimeState}
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
            appOptions={appOptions}
            feedbackMessage={feedbackMessage}
            onClearPad={handleClearPad}
            onSavePad={handleSavePad}
            onTestAction={handleTestAction}
            pad={selectedPad}
            shortcutCapability={localRuntimeState.capabilities.shortcut}
          />
        </div>
      )}
    </section>
  );
}
