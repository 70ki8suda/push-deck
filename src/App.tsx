import {
  startTransition,
  useEffect,
  useEffectEvent,
  useState,
} from "react";
import type { CSSProperties } from "react";
import { EditorPage } from "./features/editor/EditorPage";
import {
  loadCurrentConfig,
  refreshRuntimeState,
  restoreDefaultConfig,
  subscribeRuntimeEvent,
} from "./lib/api";
import type {
  Config,
  ConfigRecoveryState,
  CurrentConfigResponse,
  RestoreDefaultConfigResponse,
  RuntimeEvent,
  RuntimeState,
} from "./lib/types";

const defaultRuntimeState: RuntimeState = {
  app_state: "starting",
  capabilities: {
    shortcut: "unavailable",
  },
};

const appStyles = {
  page: {
    background:
      "radial-gradient(circle at top left, rgba(88, 140, 123, 0.24) 0%, transparent 26%), radial-gradient(circle at top right, rgba(212, 149, 72, 0.22) 0%, transparent 24%), linear-gradient(180deg, #0c0f0d 0%, #131815 44%, #181d19 100%)",
    color: "#f4f0e8",
    minHeight: "100vh",
    padding: "2rem",
  },
  shell: {
    margin: "0 auto",
    maxWidth: "88rem",
  },
  header: {
    display: "grid",
    gap: "0.35rem",
    marginBottom: "1.5rem",
  },
  eyebrow: {
    color: "#9bb09d",
    fontSize: "0.78rem",
    letterSpacing: "0.16em",
    margin: 0,
    textTransform: "uppercase",
  },
  titleRow: {
    alignItems: "baseline",
    display: "flex",
    flexWrap: "wrap",
    gap: "0.8rem",
    justifyContent: "space-between",
  },
  title: {
    fontFamily: "\"Avenir Next\", \"Segoe UI\", sans-serif",
    fontSize: "clamp(2.2rem, 5vw, 4.8rem)",
    lineHeight: 0.94,
    margin: 0,
  },
  subtitle: {
    color: "#b8c4b9",
    margin: 0,
    maxWidth: "44rem",
  },
  loadError: {
    background: "rgba(155, 63, 45, 0.18)",
    border: "1px solid rgba(204, 95, 71, 0.38)",
    borderRadius: "1rem",
    color: "#ffd5cb",
    margin: "0 0 1.2rem",
    padding: "0.95rem 1rem",
  },
} satisfies Record<string, CSSProperties>;

function selectInitialPad(config: Config | null) {
  const profile =
    config?.profiles.find(
      (candidate) => candidate.id === config.settings.activeProfileId,
    ) ?? config?.profiles[0];

  return profile?.pads[0]?.padId ?? null;
}

export function deriveLoadedState(response: CurrentConfigResponse) {
  if (response.status === "ready") {
    return {
      config: response.config,
      deviceName: response.device_name,
      isDeviceConnected: response.device_connected,
      recovery: null,
      runtimeState: response.runtime_state,
      selectedPadId: selectInitialPad(response.config),
    };
  }

  return {
    config: null,
    deviceName: response.device_name,
    isDeviceConnected: response.device_connected,
    recovery: response.recovery,
    runtimeState: response.runtime_state,
    selectedPadId: null,
  };
}

export function deriveRestoredState(response: RestoreDefaultConfigResponse) {
  return {
    config: response.config,
    recovery: null as ConfigRecoveryState | null,
    runtimeState: response.runtime_state,
    selectedPadId: selectInitialPad(response.config),
  };
}

type AppUiState = {
  config: Config | null;
  deviceName: string | null;
  isDeviceConnected: boolean;
  recovery: ConfigRecoveryState | null;
  runtimeState: RuntimeState;
  selectedPadId: string | null;
};

export function deriveRuntimeRefreshState(
  current: AppUiState,
  response: CurrentConfigResponse,
) {
  return {
    config: current.config,
    deviceName: response.device_name,
    isDeviceConnected: response.device_connected,
    recovery: response.status === "recovery_required" ? response.recovery : current.recovery,
    runtimeState: response.runtime_state,
    selectedPadId: current.selectedPadId,
  };
}

type RuntimeBootstrapDeps = {
  loadCurrentConfig: typeof loadCurrentConfig;
  subscribeRuntimeEvent: typeof subscribeRuntimeEvent;
  applyLoadedState: (response: CurrentConfigResponse) => void;
  handleRuntimeEvent: (event: RuntimeEvent) => void;
  handleLoadError: (error: unknown) => void;
};

type RuntimeRefreshDeps = {
  refreshRuntimeState: typeof refreshRuntimeState;
  applyRefreshedState: (response: CurrentConfigResponse) => void;
  handleLoadError: (error: unknown) => void;
  intervalMs?: number;
};

export function createRuntimeSubscription(deps: RuntimeBootstrapDeps) {
  let isCancelled = false;
  let cleanup: (() => void) | undefined;

  void (async () => {
    try {
      const response = await deps.loadCurrentConfig();
      if (isCancelled) {
        return;
      }

      deps.applyLoadedState(response);

      const nextCleanup = await deps.subscribeRuntimeEvent((event) => {
        if (!isCancelled) {
          deps.handleRuntimeEvent(event);
        }
      });

      if (isCancelled) {
        nextCleanup();
        return;
      }

      cleanup = nextCleanup;
    } catch (error) {
      if (!isCancelled) {
        deps.handleLoadError(error);
      }
    }
  })();

  return () => {
    isCancelled = true;
    cleanup?.();
  };
}

export function createRuntimeRefreshLoop(deps: RuntimeRefreshDeps) {
  let inFlight = false;

  const intervalId = globalThis.setInterval(async () => {
    if (inFlight) {
      return;
    }

    inFlight = true;

    try {
      deps.applyRefreshedState(await deps.refreshRuntimeState());
    } catch (error) {
      deps.handleLoadError(error);
    } finally {
      inFlight = false;
    }
  }, deps.intervalMs ?? 2000);

  return () => {
    globalThis.clearInterval(intervalId);
  };
}

export default function App() {
  const [config, setConfig] = useState<Config | null>(null);
  const [recovery, setRecovery] = useState<ConfigRecoveryState | null>(null);
  const [runtimeState, setRuntimeState] =
    useState<RuntimeState>(defaultRuntimeState);
  const [selectedPadId, setSelectedPadId] = useState<string | null>(null);
  const [deviceName, setDeviceName] = useState<string | null>(null);
  const [isDeviceConnected, setIsDeviceConnected] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);

  const applyLoadedState = useEffectEvent((response: CurrentConfigResponse) => {
    const next = deriveLoadedState(response);

    startTransition(() => {
      setConfig(next.config);
      setDeviceName(next.deviceName);
      setIsDeviceConnected(next.isDeviceConnected);
      setRecovery(next.recovery);
      setRuntimeState(next.runtimeState);
      setSelectedPadId(next.selectedPadId);
      setLoadError(null);
    });
  });

  const handleRuntimeEvent = useEffectEvent((event: RuntimeEvent) => {
    switch (event.type) {
      case "state_changed":
        startTransition(() => {
          setRuntimeState(event.state);
        });
        break;
      case "device_connection_changed":
        startTransition(() => {
          setDeviceName(event.device_name);
          setIsDeviceConnected(event.connected);
        });
        break;
      default:
        break;
    }
  });

  const applyRefreshedState = useEffectEvent((response: CurrentConfigResponse) => {
    const next = deriveRuntimeRefreshState(
      {
        config,
        deviceName,
        isDeviceConnected,
        recovery,
        runtimeState,
        selectedPadId,
      },
      response,
    );

    startTransition(() => {
      setDeviceName(next.deviceName);
      setIsDeviceConnected(next.isDeviceConnected);
      setRecovery(next.recovery);
      setRuntimeState(next.runtimeState);
      setLoadError(null);
    });
  });

  useEffect(() => {
    return createRuntimeSubscription({
      loadCurrentConfig,
      subscribeRuntimeEvent,
      applyLoadedState,
      handleRuntimeEvent,
      handleLoadError(error) {
        startTransition(() => {
          setRuntimeState((current) => ({
            ...current,
            app_state: "save_failed",
          }));
          setLoadError(
            error instanceof Error ? error.message : "Unable to load Push Deck state.",
          );
        });
      },
    });
  }, [applyLoadedState, handleRuntimeEvent]);

  useEffect(() => {
    return createRuntimeRefreshLoop({
      refreshRuntimeState,
      applyRefreshedState,
      handleLoadError(error) {
        startTransition(() => {
          setLoadError(
            error instanceof Error
              ? error.message
              : "Unable to refresh Push Deck runtime state.",
          );
        });
      },
    });
  }, [applyRefreshedState]);

  async function handleRestoreDefaultConfig() {
    try {
      const response = await restoreDefaultConfig();
      const next = deriveRestoredState(response);

      startTransition(() => {
        setConfig(next.config);
        setRecovery(next.recovery);
        setRuntimeState(next.runtimeState);
        setSelectedPadId(next.selectedPadId);
        setLoadError(null);
      });
    } catch (error) {
      try {
        await refreshRuntimeSnapshot();
      } catch {
        // Keep the existing UI state if the snapshot refresh fails too.
      }
      startTransition(() => {
        setLoadError(
          error instanceof Error
            ? error.message
            : "Unable to restore the default Push Deck layout.",
        );
      });
    }
  }

  async function refreshRuntimeSnapshot() {
    const response = await loadCurrentConfig();
    const next = deriveLoadedState(response);

    startTransition(() => {
      setConfig(next.config);
      setDeviceName(next.deviceName);
      setIsDeviceConnected(next.isDeviceConnected);
      setRecovery(next.recovery);
      setRuntimeState(next.runtimeState);
      setSelectedPadId(next.selectedPadId);
      setLoadError(null);
    });
  }

  return (
    <main style={appStyles.page}>
      <div style={appStyles.shell}>
        <header style={appStyles.header}>
          <p style={appStyles.eyebrow}>Push Deck</p>
          <div style={appStyles.titleRow}>
            <h1 style={appStyles.title}>Editor Shell</h1>
            <p style={appStyles.subtitle}>
              Shape the Push 3 grid from one runtime-aware workspace, with
              recovery gating preserved when the config store reports a broken
              layout file.
            </p>
          </div>
        </header>

        {loadError ? <p style={appStyles.loadError}>{loadError}</p> : null}

        <EditorPage
          config={config}
          runtimeState={runtimeState}
          recovery={recovery}
          selectedPadId={selectedPadId}
          deviceName={deviceName}
          isDeviceConnected={isDeviceConnected}
          onRestoreDefaultConfig={() => {
            void handleRestoreDefaultConfig();
          }}
          onRuntimeRefreshRequested={refreshRuntimeSnapshot}
          onSelectPad={setSelectedPadId}
        />
      </div>
    </main>
  );
}
