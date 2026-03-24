import { startTransition, useEffect, useState } from "react";
import type { CSSProperties } from "react";
import {
  previewPush3Palette,
  syncPush3Leds,
  triggerTestAction,
  updatePadBinding,
  updatePush3ColorCalibration,
} from "../../lib/api";
import type {
  AppPickerOption,
  Config,
  ConfigRecoveryState,
  DetailPadDraft,
  PadBinding,
  PadColorId,
  RuntimeState,
  TestActionResponse,
  UpdatePadBindingResponse,
  UpdatePush3ColorCalibrationResponse,
} from "../../lib/types";
import { DetailPanel, buildPadBindingFromDraft, clearPadBinding } from "./DetailPanel";
import { GridView } from "./GridView";
import { Push3CalibrationPanel } from "./Push3CalibrationPanel";
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

function shouldShowPush3Calibration() {
  return import.meta.env.VITE_SHOW_PUSH3_CALIBRATION === "true";
}

export interface EditorPageProps {
  config: Config | null;
  runtimeState: RuntimeState;
  recovery: ConfigRecoveryState | null;
  selectedPadId: string | null;
  appOptions?: readonly AppPickerOption[];
  deviceName: string | null;
  isDeviceConnected: boolean;
  onRuntimeRefreshRequested?: () => Promise<void> | void;
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

export function swapPadBindings(
  config: Config,
  sourcePadId: string,
  targetPadId: string,
): Config {
  const activeProfileIndex = config.profiles.findIndex(
    (profile) => profile.id === config.settings.activeProfileId,
  );
  const profileIndex = activeProfileIndex >= 0 ? activeProfileIndex : 0;
  const profile = config.profiles[profileIndex];
  if (!profile) {
    return config;
  }

  const sourceBinding = profile.pads.find((pad) => pad.padId === sourcePadId);
  const targetBinding = profile.pads.find((pad) => pad.padId === targetPadId);
  if (!sourceBinding || !targetBinding) {
    return config;
  }

  const swappedPads = profile.pads.map((pad) => {
    if (pad.padId === sourcePadId) {
      return {
        ...targetBinding,
        padId: sourcePadId,
      };
    }

    if (pad.padId === targetPadId) {
      return {
        ...sourceBinding,
        padId: targetPadId,
      };
    }

    return pad;
  });

  const profiles = [...config.profiles];
  profiles[profileIndex] = {
    ...profile,
    pads: swappedPads,
  };

  return {
    ...config,
    profiles,
  };
}

export async function persistPadBindingSwap({
  config,
  sourcePadId,
  targetPadId,
  updatePadBinding: updatePadBindingCommand = updatePadBinding,
}: {
  config: Config;
  sourcePadId: string;
  targetPadId: string;
  updatePadBinding?: (
    request: { pad_id: string; binding: PadBinding },
  ) => Promise<UpdatePadBindingResponse>;
}) {
  const nextConfig = swapPadBindings(config, sourcePadId, targetPadId);
  const profile = getActiveProfile(nextConfig);
  const nextSourceBinding = profile?.pads.find((pad) => pad.padId === sourcePadId);
  const nextTargetBinding = profile?.pads.find((pad) => pad.padId === targetPadId);

  if (!nextSourceBinding || !nextTargetBinding) {
    return {
      config,
      runtimeState: DEFAULT_RUNTIME_STATE_FALLBACK,
      selectedPadId: targetPadId,
    };
  }

  const firstResponse = await updatePadBindingCommand({
    pad_id: sourcePadId,
    binding: nextSourceBinding,
  });
  const secondResponse = await updatePadBindingCommand({
    pad_id: targetPadId,
    binding: nextTargetBinding,
  });

  return {
    config: secondResponse.config ?? firstResponse.config,
    runtimeState: secondResponse.runtime_state,
    selectedPadId: targetPadId,
  };
}

async function runTestAction(
  padId: string,
  triggerTestActionCommand = triggerTestAction,
) {
  const response: TestActionResponse = await triggerTestActionCommand(padId);
  return response.runtime_state;
}

const DEFAULT_RUNTIME_STATE_FALLBACK: RuntimeState = {
  app_state: "ready",
  capabilities: {
    shortcut: "available",
  },
};

function applyCalibrationUpdate(
  config: Config,
  logicalColor: Exclude<PadColorId, "off">,
  outputValue: number,
): Config {
  return {
    ...config,
    settings: {
      ...config.settings,
      push3ColorCalibration: {
        ...config.settings.push3ColorCalibration,
        [logicalColor]: outputValue,
      },
    },
  };
}

export function EditorPage({
  config,
  runtimeState,
  recovery,
  selectedPadId,
  appOptions = [],
  deviceName,
  isDeviceConnected,
  onRuntimeRefreshRequested,
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
  const showPush3Calibration = shouldShowPush3Calibration();

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
      await onRuntimeRefreshRequested?.();
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
      await onRuntimeRefreshRequested?.();
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

  async function handleUpdateCalibration(
    logicalColor: Exclude<PadColorId, "off">,
    outputValue: number,
  ) {
    if (localConfig === null) {
      return;
    }

    const previousConfig = localConfig;
    const optimisticConfig = applyCalibrationUpdate(
      localConfig,
      logicalColor,
      outputValue,
    );

    startTransition(() => {
      setLocalConfig(optimisticConfig);
    });

    try {
      const response: UpdatePush3ColorCalibrationResponse =
        await updatePush3ColorCalibration({
          logical_color: logicalColor,
          output_value: outputValue,
        });

      startTransition(() => {
        setLocalConfig(response.config);
        setLocalRuntimeState(response.runtime_state);
        setFeedbackMessage(`Calibrated ${logicalColor} to #${outputValue}.`);
      });
    } catch (error) {
      startTransition(() => {
        setLocalConfig(previousConfig);
        setFeedbackMessage(
          error instanceof Error
            ? error.message
            : "Unable to update Push 3 calibration.",
        );
      });
    }
  }

  async function handleMovePad(sourcePadId: string, targetPadId: string) {
    if (localConfig === null || sourcePadId === targetPadId) {
      return;
    }

    try {
      const next = await persistPadBindingSwap({
        config: localConfig,
        sourcePadId,
        targetPadId,
      });

      startTransition(() => {
        setLocalConfig(next.config);
        setLocalRuntimeState(next.runtimeState);
        setFeedbackMessage(`Moved ${sourcePadId} to ${targetPadId}.`);
        onSelectPad(next.selectedPadId);
      });
    } catch (error) {
      await onRuntimeRefreshRequested?.();
      startTransition(() => {
        setFeedbackMessage(
          error instanceof Error ? error.message : "Unable to move this pad binding.",
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
      {!isRecoveryMode && localConfig !== null && showPush3Calibration ? (
        <Push3CalibrationPanel
          calibration={localConfig.settings.push3ColorCalibration}
          onUpdateCalibration={handleUpdateCalibration}
          onPreviewPage={(page) => previewPush3Palette({ page })}
          onRestoreLayout={() => syncPush3Leds()}
        />
      ) : null}
      {isRecoveryMode ? (
        <RecoveryPanel recovery={recovery} onRestoreDefaultConfig={onRestoreDefaultConfig} />
      ) : (
        <div style={shellStyles.workspace}>
          <GridView
            pads={profile?.pads ?? []}
            selectedPadId={effectiveSelectedPadId}
            onMovePad={handleMovePad}
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
