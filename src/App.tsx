import {
  CSSProperties,
  FormEvent,
  KeyboardEvent as ReactKeyboardEvent,
  PointerEvent as ReactPointerEvent,
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  disable as disableAutostart,
  enable as enableAutostart,
  isEnabled as isAutostartEnabled,
} from "@tauri-apps/plugin-autostart";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { FitAddon } from "@xterm/addon-fit";
import { readText as readClipboardText, writeText as writeClipboardText } from "@tauri-apps/plugin-clipboard-manager";
import { Terminal } from "@xterm/xterm";
import { readTerminalPreviewCandidate, retainTerminalPreview, type TerminalPreview } from "./terminalPreview";
import cliHubIcon from "./assets/cli-hub.png";
import "@xterm/xterm/css/xterm.css";
import "./App.css";
import { DEFAULT_TERMINAL_SHORTCUTS, shortcutFromEvent, conflictingShortcut, terminalNewlineData, type TerminalShortcuts } from "./terminalShortcuts";
import { SESSION_HISTORY_STORAGE_KEY, newSessionHistoryKey, readSessionHistory, reconcileSessionHistory, sessionHistoryJson } from "./sessionHistory";
import {
  ColorModePreference,
  DEFAULT_UI_SETTINGS,
  LanguagePreference,
  resolveColorMode,
  resolveLanguage,
  SessionGrouping,
  UI_COPY,
  UiSettings,
} from "./i18n";

type SessionStatus = "idle" | "starting" | "running" | "exited" | "failed";
type LaunchPreset =
  | "powerShell"
  | "windowsPowerShell"
  | "cmd"
  | "codex"
  | "claude"
  | "custom";

interface StartTerminalResponse {
  sessionId: number;
  processId: number | null;
  pageId: string;
  preset: LaunchPreset;
  name: string;
  workingDirectory: string;
  workingDirectoryKey: string;
}

interface TerminalOutput {
  sessionId: number;
  offset: number;
  bytes: number[];
}

interface ActiveTerminalSnapshot {
  sessionId: number;
  processId: number | null;
  pageId: string;
  preset: LaunchPreset;
  name: string;
  workingDirectory: string;
  workingDirectoryKey: string;
  output: number[];
  outputStartOffset: number;
  nextOutputOffset: number;
  outputTruncated: boolean;
}

interface TerminalExit {
  sessionId: number;
  exitCode: number | null;
  error: string | null;
}

interface EnvironmentVariable {
  name: string;
  value: string;
}

interface LaunchProfile {
  id: string;
  label: string;
  preset: LaunchPreset;
  sessionName: string;
  executable: string;
  argumentsText: string;
  workingDirectory: string;
}

interface AppPreferences {
  version: number;
  launchProfiles: LaunchProfile[];
  recentDirectories: string[];
  uiSettings: UiSettings;
  workspacePages: WorkspacePage[];
  activeWorkspaceId: string;
}

interface WorkspacePage {
  id: string;
  name: string;
}

interface LoadedPreferences {
  exists: boolean;
  preferences: AppPreferences;
}

interface SessionRecord {
  historyKey: string;
  displayId?: number;
  historical?: boolean;
  id: number;
  processId: number | null;
  pageId: string;
  preset: LaunchPreset;
  name: string;
  workingDirectory: string;
  workingDirectoryKey: string;
  preview: string;
  lastOutputAt: number | null;
  status: "running" | "exited" | "failed";
  exitCode: number | null;
  error: string | null;
  recovered: boolean;
}

interface TerminalHandle {
  terminal: Terminal;
  fitAddon: FitAddon;
}

interface PendingOutput {
  chunks: Array<{ offset: number; bytes: Uint8Array }>;
  byteLength: number;
  truncated: boolean;
}

interface SessionGroup {
  key: string;
  label: string;
  title?: string;
  showHeader: boolean;
  sessions: SessionRecord[];
}

type StopConfirmation =
  | { kind: "selected"; sessionId: number; sessionName: string }
  | {
      kind: "workspace";
      pageId: string;
      pageName: string;
      count: number;
      closeAfterStop: boolean;
    };

type UpdateActivity = "idle" | "checking" | "available" | "latest" | "downloading";

const MAX_ACTIVE_SESSIONS_PER_WORKSPACE = 8;
const MAX_TOTAL_ACTIVE_SESSIONS = 24;
const MAX_WORKSPACE_PAGES = 20;
const MAX_PENDING_OUTPUT_BYTES = 256 * 1024;
const MAX_LAUNCH_PROFILES = 20;
const MAX_RECENT_DIRECTORIES = 10;
const MAX_ATTACHMENTS = 10;
const QUIET_AFTER_MS = 60_000;
const SUBMIT_AFTER_INPUT_DELAY_MS = 160;
const SESSION_SIDEBAR_MIN_WIDTH = 190;
const SESSION_SIDEBAR_DEFAULT_WIDTH = 360;
const SESSION_SIDEBAR_MAX_WIDTH = 360;
const SESSION_SIDEBAR_MAX_RATIO = 0.4;
const TERMINAL_STAGE_MIN_WIDTH = 320;
const INPUT_PANEL_MIN_HEIGHT = 160;
const INPUT_PANEL_DEFAULT_HEIGHT = 166;
const INPUT_PANEL_MAX_HEIGHT = 360;
const INPUT_PANEL_MAX_RATIO = 0.45;
const PANEL_RESIZER_SIZE = 8;
const LAUNCH_PROFILES_STORAGE_KEY = "cli-hub.launch-profiles.v1";
const RECENT_DIRECTORIES_STORAGE_KEY = "cli-hub.recent-directories.v1";
const UI_SETTINGS_STORAGE_KEY = "cli-hub.ui-settings.v1";
const NEWLINE_FALLBACK_STORAGE_KEY = "cli-hub.newline-shortcut.v1";
const TERMINAL_FONT_FAMILY =
  '"CLI Hub JetBrainsMono NF", "CLI Hub Sarasa Mono SC", "Cascadia Mono", "Cascadia Code", Consolas, monospace';
const DEFAULT_WORKSPACE: WorkspacePage = {
  id: "workspace-default",
  name: "Workspace 1",
};

const presetDefaults: Record<LaunchPreset, { name: string; executable: string }> = {
  powerShell: {
    name: "PowerShell",
    executable: "Auto-detect: pwsh.exe → powershell.exe",
  },
  windowsPowerShell: {
    name: "Windows PowerShell",
    executable: "powershell.exe",
  },
  cmd: {
    name: "Command Prompt",
    executable: "cmd.exe",
  },
  codex: {
    name: "Codex",
    executable: "Auto-detect: codex.exe / codex.ps1 / codex.cmd",
  },
  claude: {
    name: "Claude Code",
    executable: "Auto-detect: claude.exe / claude.ps1 / claude.cmd",
  },
  custom: {
    name: "Custom CLI",
    executable: "",
  },
};

// Templates use the existing Custom CLI contract so running hosts and saved
// profiles remain compatible without introducing new backend preset variants.
const agentTemplates = {
  pi: { name: "Pi", executable: "pi" },
  hermes: { name: "Hermes Agent", executable: "hermes" },
  opencode: { name: "OpenCode", executable: "opencode" },
} as const;
type AgentTemplate = keyof typeof agentTemplates;

function App() {
  const [sessionHistoryReady, setSessionHistoryReady] = useState(false);
  const terminalHandlesRef = useRef(new Map<number, TerminalHandle>());
  const knownSessionIdsRef = useRef(new Set<number>());
  const discardedSessionIdsRef = useRef(new Set<number>());
  const outputCacheRef = useRef(new Map<number, PendingOutput>());
  const pendingExitRef = useRef(new Map<number, TerminalExit>());
  const previewTimersRef = useRef(new Map<number, number>());
  const activityTimersRef = useRef(new Map<number, number>());
  const retainedPreviewsRef = useRef(new Map<number, TerminalPreview>());
  const nextOutputOffsetRef = useRef(new Map<number, number>());
  const lastOutputAtRef = useRef(new Map<number, number>());
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const terminalCardRef = useRef<HTMLElement>(null);
  const sidebarOverviewRef = useRef<HTMLDivElement>(null);
  const sidebarCompactRef = useRef<HTMLDivElement>(null);
  const sidebarScrollPositionsRef = useRef(new Map<string, number>());
  const workspaceStackRef = useRef<HTMLDivElement>(null);
  const inputPanelRef = useRef<HTMLFormElement>(null);
  const pendingInputCaretRef = useRef<number | null>(null);
  const autostartVisibilityHandledRef = useRef(false);

  const [sessions, setSessions] = useState<SessionRecord[]>([]);
  const savedSessionHistoryJson = sessionHistoryJson(sessions);
  const latestSessionHistoryRef = useRef(savedSessionHistoryJson);
  latestSessionHistoryRef.current = savedSessionHistoryJson;
  useEffect(() => {
    if (!sessionHistoryReady) return;
    try {
      localStorage.setItem(SESSION_HISTORY_STORAGE_KEY, savedSessionHistoryJson);
    } catch (reason) {
      setError(`Unable to save session records: ${describeError(reason)}`);
    }
  }, [savedSessionHistoryJson, sessionHistoryReady]);
  const [workspacePages, setWorkspacePages] = useState<WorkspacePage[]>([DEFAULT_WORKSPACE]);
  const [activeWorkspaceId, setActiveWorkspaceId] = useState(DEFAULT_WORKSPACE.id);
  const [editingWorkspaceId, setEditingWorkspaceId] = useState<string | null>(null);
  const [workspaceNameDraft, setWorkspaceNameDraft] = useState("");
  const [selectedSessionId, setSelectedSessionId] = useState<number | null>(null);
  const [terminalFocusMode, setTerminalFocusMode] = useState(false);
  const [listenersReady, setListenersReady] = useState(false);
  const [launcherOpen, setLauncherOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [settingsError, setSettingsError] = useState("");
  const [appVersion, setAppVersion] = useState("");
  const [availableUpdate, setAvailableUpdate] = useState<Update | null>(null);
  const [updateActivity, setUpdateActivity] = useState<UpdateActivity>("idle");
  const [updateProgress, setUpdateProgress] = useState<number | null>(null);
  const [autostartEnabled, setAutostartEnabled] = useState(false);
  const [autostartLoading, setAutostartLoading] = useState(false);
  const [launchStarting, setLaunchStarting] = useState(false);
  const [stopConfirmation, setStopConfirmation] = useState<StopConfirmation | null>(null);
  const [stopStarting, setStopStarting] = useState(false);
  const [closeRequestCount, setCloseRequestCount] = useState<number | null>(null);
  const [shutdownStarting, setShutdownStarting] = useState(false);
  const [launchProfiles, setLaunchProfiles] = useState<LaunchProfile[]>(readLaunchProfiles);
  const [preferencesReady, setPreferencesReady] = useState(false);
  const [profileSaving, setProfileSaving] = useState(false);
  const [selectedProfileId, setSelectedProfileId] = useState("");
  const [recentDirectories, setRecentDirectories] = useState<string[]>(readRecentDirectories);
  const [preset, setPreset] = useState<LaunchPreset>("powerShell");
  const [agentTemplate, setAgentTemplate] = useState<AgentTemplate | null>(null);
  const [sessionName, setSessionName] = useState(presetDefaults.powerShell.name);
  const [executable, setExecutable] = useState(presetDefaults.powerShell.executable);
  const [argumentsText, setArgumentsText] = useState("");
  const [environmentText, setEnvironmentText] = useState("");
  const [workingDirectory, setWorkingDirectory] = useState("");
  const [input, setInput] = useState("");
  const [inputSending, setInputSending] = useState(false);
  const [attachments, setAttachments] = useState<string[]>([]);
  const [sessionSidebarWidth, setSessionSidebarWidth] = useState(
    SESSION_SIDEBAR_DEFAULT_WIDTH,
  );
  const [inputPanelHeight, setInputPanelHeight] = useState(
    INPUT_PANEL_DEFAULT_HEIGHT,
  );
  const [uiSettings, setUiSettings] = useState<UiSettings>(readUiSettings);
  const [sidebarSaving, setSidebarSaving] = useState(false);
  const [panelResizing, setPanelResizing] = useState(false);
  const [systemPrefersDark, setSystemPrefersDark] = useState(
    () => window.matchMedia("(prefers-color-scheme: dark)").matches,
  );
  const [error, setError] = useState("");
  const resolvedLanguage = resolveLanguage(uiSettings.language);
  const resolvedColorMode = resolveColorMode(uiSettings.colorMode, systemPrefersDark);
  const copy = UI_COPY[resolvedLanguage];

  useLayoutEffect(() => {
    const collapsed = uiSettings.sessionSidebarCollapsed;
    const element = collapsed ? sidebarCompactRef.current : sidebarOverviewRef.current;
    if (element) {
      element.scrollTop = sidebarScrollPositionsRef.current.get(
        `${activeWorkspaceId}:${collapsed ? "compact" : "overview"}`,
      ) ?? 0;
    }
  }, [activeWorkspaceId, uiSettings.sessionSidebarCollapsed]);

  useEffect(() => {
    let cancelled = false;

    async function initializePreferences() {
      try {
        await invoke<LoadedPreferences>("load_app_preferences");
        const preferences = await invoke<AppPreferences>("migrate_legacy_preferences", {
          origin: window.location.origin,
          preferences: {
            version: 1,
            launchProfiles,
            recentDirectories,
            uiSettings,
            workspacePages,
            activeWorkspaceId,
          } satisfies AppPreferences,
        });
        if (cancelled) {
          return;
        }

        setLaunchProfiles(preferences.launchProfiles);
        setRecentDirectories(preferences.recentDirectories);
        const terminalShortcuts = { ...DEFAULT_TERMINAL_SHORTCUTS, ...preferences.uiSettings.terminalShortcuts };
        // A live v0.1.9 host drops unknown preference fields during interface updates.
        const savedNewline = localStorage.getItem(NEWLINE_FALLBACK_STORAGE_KEY);
        if (savedNewline && !conflictingShortcut({ ...terminalShortcuts, newline: savedNewline })) {
          terminalShortcuts.newline = savedNewline;
        }
        setUiSettings({ ...DEFAULT_UI_SETTINGS, ...preferences.uiSettings, terminalShortcuts });
        setWorkspacePages(
          preferences.workspacePages.length > 0
            ? preferences.workspacePages
            : [DEFAULT_WORKSPACE],
        );
        setActiveWorkspaceId(
          preferences.workspacePages.some(
            (workspace) => workspace.id === preferences.activeWorkspaceId,
          )
            ? preferences.activeWorkspaceId
            : (preferences.workspacePages[0]?.id ?? DEFAULT_WORKSPACE.id),
        );
        clearLegacyPreferences();
        setPreferencesReady(true);
      } catch (reason) {
        if (!cancelled) {
          setError(`Unable to load saved preferences: ${describeError(reason)}`);
        }
      }
    }

    void initializePreferences();
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    document.documentElement.lang = resolvedLanguage;
  }, [resolvedLanguage]);

  useEffect(() => {
    const colorScheme = window.matchMedia("(prefers-color-scheme: dark)");
    const updateSystemColorMode = (event: MediaQueryListEvent) => {
      setSystemPrefersDark(event.matches);
    };

    colorScheme.addEventListener("change", updateSystemColorMode);
    return () => colorScheme.removeEventListener("change", updateSystemColorMode);
  }, []);

  useEffect(() => {
    document.documentElement.dataset.colorMode = resolvedColorMode;
    document.documentElement.style.colorScheme = resolvedColorMode;
  }, [resolvedColorMode]);

  useEffect(() => {
    if (!preferencesReady || autostartVisibilityHandledRef.current) {
      return;
    }
    autostartVisibilityHandledRef.current = true;
    let cancelled = false;

    void invoke<boolean>("is_autostart_launch")
      .then((autostartLaunch) => {
        if (!cancelled && autostartLaunch && uiSettings.startMinimized) {
          return getCurrentWindow().hide();
        }
        return undefined;
      })
      .catch(() => undefined);

    return () => {
      cancelled = true;
    };
  }, [preferencesReady, uiSettings.startMinimized]);

  useEffect(() => {
    const inputElement = inputRef.current;
    if (!inputElement) {
      return;
    }

    inputElement.style.height = "auto";
    inputElement.style.height = `${Math.min(inputElement.scrollHeight, 136)}px`;
    if (pendingInputCaretRef.current !== null) {
      inputElement.setSelectionRange(
        pendingInputCaretRef.current,
        pendingInputCaretRef.current,
      );
      pendingInputCaretRef.current = null;
    }
  }, [input]);

  useEffect(() => {
    if (terminalFocusMode && !selectedSessionId) {
      setTerminalFocusMode(false);
    }
  }, [selectedSessionId, terminalFocusMode]);

  useEffect(() => {
    if (!launcherOpen) {
      return;
    }

    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !launchStarting) {
        setLauncherOpen(false);
        setError("");
      }
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [launcherOpen, launchStarting]);

  useEffect(() => {
    if (!settingsOpen) {
      return;
    }

    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !autostartLoading) {
        setSettingsOpen(false);
        setSettingsError("");
      }
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [settingsOpen, autostartLoading]);

  useEffect(() => {
    if (!stopConfirmation) {
      return;
    }

    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !stopStarting) {
        setStopConfirmation(null);
        setError("");
      }
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [stopConfirmation, stopStarting]);

  const schedulePreview = useCallback((sessionId: number, terminal: Terminal) => {
    // Activity remains live while the preview waits for output to settle.
    if (!activityTimersRef.current.has(sessionId)) {
      const activityTimer = window.setTimeout(() => {
        activityTimersRef.current.delete(sessionId);
        const lastOutputAt = lastOutputAtRef.current.get(sessionId) ?? null;
        setSessions((current) => current.map((session) =>
          session.id === sessionId && session.lastOutputAt !== lastOutputAt
            ? { ...session, lastOutputAt } : session));
      }, 500);
      activityTimersRef.current.set(sessionId, activityTimer);
    }

    const previousTimer = previewTimersRef.current.get(sessionId);
    if (previousTimer !== undefined) window.clearTimeout(previousTimer);
    const timer = window.setTimeout(() => {
      previewTimersRef.current.delete(sessionId);
      const retained = retainTerminalPreview(retainedPreviewsRef.current.get(sessionId), readTerminalPreviewCandidate(terminal));
      retainedPreviewsRef.current.set(sessionId, retained);
      const preview = retained.text || "Waiting for terminal output…";
      const lastOutputAt = lastOutputAtRef.current.get(sessionId) ?? null;
      setSessions((current) =>
        current.map((session) => {
          if (session.id !== sessionId) {
            return session;
          }
          return session.preview !== preview || session.lastOutputAt !== lastOutputAt
            ? { ...session, preview, lastOutputAt }
            : session;
        }),
      );
    }, 2_000);
    previewTimersRef.current.set(sessionId, timer);
  }, []);

  const writeTerminalOutput = useCallback(
    (terminal: Terminal, output: TerminalOutput) => {
      const result = takeUnseenOutput(nextOutputOffsetRef.current, output);
      if (!result) {
        return;
      }
      if (result.gap) {
        terminal.writeln("\r\n[CLI Hub] Some terminal output was unavailable after refresh.\r");
      }
      terminal.write(result.bytes, () => schedulePreview(output.sessionId, terminal));
    },
    [schedulePreview],
  );

  useEffect(() => {
    let disposed = false;
    const unlisteners: UnlistenFn[] = [];

    const registerListeners = async () => {
      const outputUnlisten = await listen<TerminalOutput>(
        "terminal-output",
        ({ payload }) => {
          if (discardedSessionIdsRef.current.has(payload.sessionId)) {
            return;
          }
          lastOutputAtRef.current.set(payload.sessionId, Date.now());
          queuePendingOutput(outputCacheRef.current, payload);
          const handle = terminalHandlesRef.current.get(payload.sessionId);
          if (handle) {
            writeTerminalOutput(handle.terminal, payload);
          }
        },
      );
      if (disposed) {
        outputUnlisten();
        return;
      }
      unlisteners.push(outputUnlisten);

      const exitUnlisten = await listen<TerminalExit>("terminal-exit", ({ payload }) => {
        if (discardedSessionIdsRef.current.has(payload.sessionId)) {
          return;
        }
        const handle = terminalHandlesRef.current.get(payload.sessionId);
        if (handle) {
          writeExitNotice(handle.terminal, payload);
          schedulePreview(payload.sessionId, handle.terminal);
        } else {
          pendingExitRef.current.set(payload.sessionId, payload);
        }

        if (knownSessionIdsRef.current.has(payload.sessionId)) {
          setSessions((current) =>
            current.map((session) =>
              session.id === payload.sessionId
                ? {
                    ...session,
                    status: payload.error ? "failed" : "exited",
                    exitCode: payload.exitCode,
                    error: payload.error,
                  }
                : session,
            ),
          );
        } else {
          pendingExitRef.current.set(payload.sessionId, payload);
        }
      });
      if (disposed) {
        exitUnlisten();
        return;
      }
      unlisteners.push(exitUnlisten);

      const closeUnlisten = await listen<number>(
        "application-close-requested",
        ({ payload }) => {
          setTerminalFocusMode(false);
          setLauncherOpen(false);
          setStopConfirmation(null);
          setError("");
          setCloseRequestCount(payload);
        },
      );
      if (disposed) {
        closeUnlisten();
        return;
      }
      unlisteners.push(closeUnlisten);

      const persistenceUnlisten = await listen<string>("session-persistence-error", ({ payload }) => setError(payload));
      if (disposed) { persistenceUnlisten(); return; }
      unlisteners.push(persistenceUnlisten);
      const hostUnlisten = await listen<string>("terminal-host-disconnected", ({ payload }) => {
        setError(payload);
        setListenersReady(false);
      });
      if (disposed) { hostUnlisten(); return; }
      unlisteners.push(hostUnlisten);
      const restoreErrors = await invoke<string[]>("restore_terminals");
      if (restoreErrors?.length) setError(restoreErrors.join("\n"));
      const activeSessions = await invoke<ActiveTerminalSnapshot[]>("list_active_terminals");
      if (disposed) {
        return;
      }

      let savedHistory: ReturnType<typeof readSessionHistory> = [];
      let historyLoaded = true;
      try {
        savedHistory = readSessionHistory(localStorage);
      } catch (reason) {
        historyLoaded = false;
        setError(`Unable to load session records: ${describeError(reason)}`);
      }
      const recoveredSessions = activeSessions.map((snapshot) => {
        knownSessionIdsRef.current.add(snapshot.sessionId);
        if (snapshot.output.length > 0) {
          queuePendingOutput(
            outputCacheRef.current,
            {
              sessionId: snapshot.sessionId,
              offset: snapshot.outputStartOffset,
              bytes: snapshot.output,
            },
            snapshot.outputTruncated,
          );
        } else {
          nextOutputOffsetRef.current.set(snapshot.sessionId, snapshot.nextOutputOffset);
        }

        const earlyExit = pendingExitRef.current.get(snapshot.sessionId);
        return {
          historyKey: newSessionHistoryKey(),
          id: snapshot.sessionId,
          processId: snapshot.processId,
          pageId: snapshot.pageId,
          preset: snapshot.preset,
          name: snapshot.name,
          workingDirectory: snapshot.workingDirectory,
          workingDirectoryKey: snapshot.workingDirectoryKey,
          preview: snapshot.outputTruncated
            ? "Recovered recent output…"
            : "Recovering terminal output…",
          lastOutputAt: lastOutputAtRef.current.get(snapshot.sessionId) ?? null,
          status: earlyExit ? (earlyExit.error ? "failed" : "exited") : "running",
          exitCode: earlyExit?.exitCode ?? null,
          error: earlyExit?.error ?? null,
          recovered: true,
        } satisfies SessionRecord;
      });

      const reconciled = reconcileSessionHistory(savedHistory, recoveredSessions);
      const historyCards: SessionRecord[] = reconciled.ended.map((record, index) => ({
        ...record, id: -index - 1, displayId: record.id, historical: true,
        preview: "", lastOutputAt: null, error: null, recovered: false,
      }));
      const liveCards = recoveredSessions.map((record, index) => ({
        ...record, historyKey: reconciled.active[index].historyKey,
      }));
      setSessions([...historyCards, ...liveCards]);
      setSelectedSessionId(liveCards[liveCards.length - 1]?.id ?? historyCards[historyCards.length - 1]?.id ?? null);
      setSessionHistoryReady(historyLoaded);
      setListenersReady(true);
    };

    void registerListeners().catch((reason) => {
      if (!disposed) {
        setError(`Unable to listen for terminal events: ${describeError(reason)}`);
      }
    });

    return () => {
      disposed = true;
      unlisteners.forEach((unlisten) => unlisten());
      previewTimersRef.current.forEach((timer) => window.clearTimeout(timer));
      previewTimersRef.current.clear();
      activityTimersRef.current.forEach((timer) => window.clearTimeout(timer));
      activityTimersRef.current.clear();
      retainedPreviewsRef.current.clear();
    };
  }, [schedulePreview, writeTerminalOutput]);

  useEffect(() => {
    const currentSelectionIsVisible = sessions.some(
      (session) =>
        session.id === selectedSessionId && session.pageId === activeWorkspaceId,
    );
    if (currentSelectionIsVisible) {
      return;
    }
    const visibleSessions = sessions.filter(
      (session) => session.pageId === activeWorkspaceId,
    );
    setSelectedSessionId(visibleSessions[visibleSessions.length - 1]?.id ?? null);
  }, [activeWorkspaceId, selectedSessionId, sessions]);

  const registerTerminal = useCallback(
    (sessionId: number, terminal: Terminal, fitAddon: FitAddon) => {
      terminalHandlesRef.current.set(sessionId, { terminal, fitAddon });
      nextOutputOffsetRef.current.delete(sessionId);

      const pendingOutput = outputCacheRef.current.get(sessionId);
      if (pendingOutput) {
        if (pendingOutput.truncated) {
          terminal.writeln("[CLI Hub] Earlier terminal output was truncated.\r");
        }
        pendingOutput.chunks
          .sort((left, right) => left.offset - right.offset)
          .forEach((chunk) =>
            writeTerminalOutput(terminal, {
              sessionId,
              offset: chunk.offset,
              bytes: Array.from(chunk.bytes),
            }),
          );
      }

      const pendingExit = pendingExitRef.current.get(sessionId);
      if (pendingExit) {
        writeExitNotice(terminal, pendingExit);
        schedulePreview(sessionId, terminal);
      }
    },
    [schedulePreview, writeTerminalOutput],
  );

  const unregisterTerminal = useCallback((sessionId: number, terminal: Terminal) => {
    if (terminalHandlesRef.current.get(sessionId)?.terminal === terminal) {
      terminalHandlesRef.current.delete(sessionId);
    }
  }, []);

  const reportTerminalError = useCallback((reason: unknown) => {
    setError(describeError(reason));
  }, []);

  function changePreset(selection: LaunchPreset | AgentTemplate) {
    const template = Object.prototype.hasOwnProperty.call(agentTemplates, selection)
      ? selection as AgentTemplate : null;
    const nextPreset = template ? "custom" : selection as LaunchPreset;
    const defaults = template ? agentTemplates[template] : presetDefaults[nextPreset];
    setAgentTemplate(template);
    setPreset(nextPreset);
    setSessionName(defaults.name);
    setExecutable(defaults.executable);
    setArgumentsText("");
    setError("");
  }

  function selectLaunchProfile(profileId: string) {
    setSelectedProfileId(profileId);
    const profile = launchProfiles.find((candidate) => candidate.id === profileId);
    if (!profile) {
      return;
    }

    setPreset(profile.preset);
    setAgentTemplate(profile.preset === "custom"
      ? (Object.keys(agentTemplates) as AgentTemplate[]).find(
        (key) => agentTemplates[key].executable === profile.executable,
      ) ?? null
      : null);
    setSessionName(profile.sessionName);
    setExecutable(profile.executable);
    setArgumentsText(profile.argumentsText);
    setWorkingDirectory(profile.workingDirectory);
    setEnvironmentText("");
    setError("");
  }

  async function saveLaunchProfile() {
    const label = sessionName.trim();
    if (!label) {
      setError("Enter a display name before saving this profile.");
      return;
    }
    if (!selectedProfileId && launchProfiles.length >= MAX_LAUNCH_PROFILES) {
      setError(`Delete a saved profile before adding more than ${MAX_LAUNCH_PROFILES}.`);
      return;
    }

    const profile: LaunchProfile = {
      id: selectedProfileId || createProfileId(),
      label,
      preset,
      sessionName: label,
      executable,
      argumentsText,
      workingDirectory: workingDirectory.trim(),
    };
    const nextProfiles = selectedProfileId
      ? launchProfiles.map((candidate) =>
          candidate.id === selectedProfileId ? profile : candidate,
        )
      : [...launchProfiles, profile];

    setProfileSaving(true);
    try {
      await invoke("save_launch_profiles", { launchProfiles: nextProfiles });
      setLaunchProfiles(nextProfiles);
      setSelectedProfileId(profile.id);
      setError("");
    } catch (reason) {
      setError(`Unable to save launch profile: ${describeError(reason)}`);
    } finally {
      setProfileSaving(false);
    }
  }

  async function deleteLaunchProfile() {
    if (!selectedProfileId) {
      return;
    }

    const nextProfiles = launchProfiles.filter((profile) => profile.id !== selectedProfileId);
    setProfileSaving(true);
    try {
      await invoke("save_launch_profiles", { launchProfiles: nextProfiles });
      setLaunchProfiles(nextProfiles);
      setSelectedProfileId("");
      setError("");
    } catch (reason) {
      setError(`Unable to delete launch profile: ${describeError(reason)}`);
    } finally {
      setProfileSaving(false);
    }
  }

  function rememberRecentDirectory(directory: string) {
    const comparisonKey = directoryComparisonKey(directory);
    const nextDirectories = [
      directory,
      ...recentDirectories.filter(
        (candidate) => directoryComparisonKey(candidate) !== comparisonKey,
      ),
    ].slice(0, MAX_RECENT_DIRECTORIES);

    setRecentDirectories(nextDirectories);
    void invoke("save_recent_directories", { recentDirectories: nextDirectories }).catch(
      (reason) => {
        setError(
          `Session started, but the recent directory could not be saved: ${describeError(reason)}`,
        );
      },
    );
  }

  async function startTerminal() {
    if (launchDisabled) {
      return;
    }

    let environment: EnvironmentVariable[];
    try {
      environment = parseEnvironment(environmentText);
    } catch (reason) {
      setError(describeError(reason));
      return;
    }

    const selectedHandle =
      selectedSessionId === null ? undefined : terminalHandlesRef.current.get(selectedSessionId);
    setLaunchStarting(true);
    setError("");

    try {
      const response = await invoke<StartTerminalResponse>("start_terminal", {
        request: {
          pageId: activeWorkspaceId,
          preset,
          name: sessionName.trim(),
          executable: preset === "custom" ? executable.trim() : "",
          arguments: parseArguments(argumentsText),
          workingDirectory: workingDirectory.trim() || null,
          environment,
        },
        cols: selectedHandle?.terminal.cols ?? 100,
        rows: selectedHandle?.terminal.rows ?? 30,
      });
      knownSessionIdsRef.current.add(response.sessionId);
      const earlyExit = pendingExitRef.current.get(response.sessionId);
      const session: SessionRecord = {
        historyKey: newSessionHistoryKey(),
        id: response.sessionId,
        processId: response.processId,
        pageId: response.pageId,
        preset: response.preset,
        name: response.name,
        workingDirectory: response.workingDirectory,
        workingDirectoryKey: response.workingDirectoryKey,
        preview: "Waiting for terminal output…",
        lastOutputAt: lastOutputAtRef.current.get(response.sessionId) ?? null,
        status: earlyExit ? (earlyExit.error ? "failed" : "exited") : "running",
        exitCode: earlyExit?.exitCode ?? null,
        error: earlyExit?.error ?? null,
        recovered: false,
      };
      setSessions((current) => [...current, session]);
      setSelectedSessionId(response.sessionId);
      rememberRecentDirectory(response.workingDirectory);
      setSelectedProfileId("");
      setEnvironmentText("");
      setInput("");
      setLauncherOpen(false);
    } catch (reason) {
      setError(describeError(reason));
    } finally {
      setLaunchStarting(false);
    }
  }

  const activeWorkspace =
    workspacePages.find((workspace) => workspace.id === activeWorkspaceId) ??
    workspacePages[0];
  const workspaceSessions = sessions.filter(
    (session) => session.pageId === activeWorkspaceId,
  );
  const selectedSession =
    workspaceSessions.find((session) => session.id === selectedSessionId) ?? null;
  const activeCount = workspaceSessions.filter(
    (session) => session.status === "running",
  ).length;
  const totalActiveCount = sessions.filter(
    (session) => session.status === "running",
  ).length;
  const selectedStatus: SessionStatus = launchStarting
    ? "starting"
    : selectedSession?.status ?? "idle";
  const isRunning = selectedSession?.status === "running";
  const launchDisabled =
    !listenersReady ||
    launchStarting ||
    activeCount >= MAX_ACTIVE_SESSIONS_PER_WORKSPACE ||
    totalActiveCount >= MAX_TOTAL_ACTIVE_SESSIONS;
  const sessionGroups = groupSessions(workspaceSessions, uiSettings.sessionGrouping);

  useEffect(() => {
    void invoke("update_tray_status", { activeCount: totalActiveCount }).catch(() => undefined);
  }, [totalActiveCount]);

  function openLauncher() {
    if (launchDisabled) {
      return;
    }
    setError("");
    setLauncherOpen(true);
  }

  function closeLauncher() {
    if (!launchStarting) {
      setError("");
      setLauncherOpen(false);
    }
  }

  function persistWorkspaceState(nextPages: WorkspacePage[], nextActiveId: string) {
    setWorkspacePages(nextPages);
    setActiveWorkspaceId(nextActiveId);
    void invoke("save_workspace_pages", {
      workspacePages: nextPages,
      activeWorkspaceId: nextActiveId,
    }).catch((reason) => setError(copy.workspaceSaveError(describeError(reason))));
  }

  function selectWorkspace(pageId: string) {
    if (pageId === activeWorkspaceId) {
      return;
    }
    const pageSessions = sessions.filter((session) => session.pageId === pageId);
    const nextSelected = pageSessions[pageSessions.length - 1]?.id ?? null;
    setTerminalFocusMode(false);
    setSelectedSessionId(nextSelected);
    setInput("");
    setAttachments([]);
    persistWorkspaceState(workspacePages, pageId);
  }

  function addWorkspace() {
    if (workspacePages.length >= MAX_WORKSPACE_PAGES) {
      setError(copy.maxWorkspaces(MAX_WORKSPACE_PAGES));
      return;
    }
    const workspace: WorkspacePage = {
      id: createWorkspaceId(),
      name: copy.workspaceName(workspacePages.length + 1),
    };
    setSelectedSessionId(null);
    setInput("");
    setAttachments([]);
    persistWorkspaceState([...workspacePages, workspace], workspace.id);
  }

  function beginWorkspaceRename(workspace: WorkspacePage) {
    setEditingWorkspaceId(workspace.id);
    setWorkspaceNameDraft(workspace.name);
  }

  function commitWorkspaceRename() {
    if (!editingWorkspaceId) {
      return;
    }
    const name = workspaceNameDraft.trim().slice(0, 40);
    if (!name) {
      setEditingWorkspaceId(null);
      return;
    }
    const nextPages = workspacePages.map((workspace) =>
      workspace.id === editingWorkspaceId ? { ...workspace, name } : workspace,
    );
    setEditingWorkspaceId(null);
    persistWorkspaceState(nextPages, activeWorkspaceId);
  }

  function requestCloseWorkspace(pageId: string) {
    if (workspacePages.length === 1) {
      return;
    }
    const workspace = workspacePages.find((candidate) => candidate.id === pageId);
    if (!workspace) {
      return;
    }
    const runningCount = sessions.filter(
      (session) => session.pageId === pageId && session.status === "running",
    ).length;
    if (runningCount > 0) {
      setStopConfirmation({
        kind: "workspace",
        pageId,
        pageName: workspace.name,
        count: runningCount,
        closeAfterStop: true,
      });
      return;
    }
    removeWorkspace(pageId);
  }

  function removeWorkspace(pageId: string) {
    const pageIndex = workspacePages.findIndex((workspace) => workspace.id === pageId);
    if (pageIndex < 0 || workspacePages.length === 1) {
      return;
    }
    const removedSessions = sessions.filter((session) => session.pageId === pageId);
    removedSessions.forEach((session) => discardSessionResources(session.id));
    setSessions((current) => current.filter((session) => session.pageId !== pageId));

    const nextPages = workspacePages.filter((workspace) => workspace.id !== pageId);
    const nextActiveId =
      activeWorkspaceId === pageId
        ? nextPages[Math.min(pageIndex, nextPages.length - 1)].id
        : activeWorkspaceId;
    const nextPageSessions = sessions.filter(
      (session) => session.pageId === nextActiveId,
    );
    const nextSelected = nextPageSessions[nextPageSessions.length - 1]?.id ?? null;
    setSelectedSessionId(nextSelected);
    persistWorkspaceState(nextPages, nextActiveId);
  }

  async function openSettings() {
    setSettingsError("");
    setSettingsOpen(true);
    setAutostartLoading(true);
    void getVersion()
      .then(setAppVersion)
      .catch((reason) => setSettingsError(copy.updateCheckError(describeError(reason))));
    try {
      setAutostartEnabled(await isAutostartEnabled());
    } catch (reason) {
      setSettingsError(copy.startupError(describeError(reason)));
    } finally {
      setAutostartLoading(false);
    }
  }

  function closeSettings() {
    if (!autostartLoading && updateActivity !== "downloading") {
      if (availableUpdate) {
        void availableUpdate.close().catch(() => undefined);
        setAvailableUpdate(null);
      }
      setUpdateActivity("idle");
      setUpdateProgress(null);
      setSettingsOpen(false);
      setSettingsError("");
    }
  }

  async function checkForUpdates() {
    setSettingsError("");
    setUpdateProgress(null);
    setUpdateActivity("checking");

    const previousUpdate = availableUpdate;
    setAvailableUpdate(null);
    if (previousUpdate) {
      await previousUpdate.close().catch(() => undefined);
    }

    try {
      const update = await check({ timeout: 30_000 });
      if (update) {
        setAvailableUpdate(update);
        setUpdateActivity("available");
      } else {
        setUpdateActivity("latest");
      }
    } catch (reason) {
      setUpdateActivity("idle");
      setSettingsError(copy.updateCheckError(describeError(reason)));
    }
  }

  async function installAvailableUpdate() {
    if (!availableUpdate || !listenersReady) return;

    setSettingsError("");
    setUpdateActivity("downloading");
    setUpdateProgress(null);
    let downloaded = 0;
    let contentLength: number | undefined;

    try {
      await availableUpdate.download((event) => {
        if (event.event === "Started") {
          contentLength = event.data.contentLength;
          downloaded = 0;
          setUpdateProgress(contentLength ? 0 : null);
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          if (contentLength) {
            setUpdateProgress(Math.min(100, Math.round((downloaded / contentLength) * 100)));
          }
        } else if (event.event === "Finished") {
          setUpdateProgress(100);
        }
      });
      if (!sessionHistoryReady) throw new Error("Session records are unavailable; update was not installed");
      localStorage.setItem(SESSION_HISTORY_STORAGE_KEY, latestSessionHistoryRef.current);
      await invoke("prepare_interface_update");
      await availableUpdate.install();
      await relaunch();
    } catch (reason) {
      setUpdateActivity("available");
      setSettingsError(copy.updateInstallError(describeError(reason)));
    }
  }

  async function changeAutostart(enabled: boolean) {
    setSettingsError("");
    setAutostartLoading(true);
    try {
      if (enabled) {
        await enableAutostart();
      } else {
        await disableAutostart();
      }
      setAutostartEnabled(await isAutostartEnabled());
    } catch (reason) {
      setSettingsError(copy.startupError(describeError(reason)));
    } finally {
      setAutostartLoading(false);
    }
  }

  async function updateUiSettings(patch: Partial<UiSettings>) {
    const next = { ...uiSettings, ...patch };
    try {
      const saved = await invoke<AppPreferences>("save_ui_settings", { uiSettings: next });
      if (saved.uiSettings.terminalShortcuts?.newline === next.terminalShortcuts.newline) {
        localStorage.removeItem(NEWLINE_FALLBACK_STORAGE_KEY);
      } else {
        localStorage.setItem(NEWLINE_FALLBACK_STORAGE_KEY, next.terminalShortcuts.newline);
      }
      setUiSettings(next);
      setSettingsError("");
    } catch (reason) {
      setSettingsError(copy.settingsSaveError(describeError(reason)));
    }
  }

  function captureShortcut(action: "copy" | "paste" | "pasteAlternate" | "interrupt" | "newline", event: ReactKeyboardEvent<HTMLInputElement>) {
    if (event.key === "Tab" || event.nativeEvent.isComposing || event.keyCode === 229) return;
    event.preventDefault();
    const shortcut = shortcutFromEvent(event.nativeEvent);
    if (!shortcut) {
      if (!["Control", "Shift", "Alt", "Meta"].includes(event.key)) setSettingsError(copy.shortcutInvalid);
      return;
    }
    const next = { ...uiSettings.terminalShortcuts, [action]: shortcut };
    if (shortcut === uiSettings.terminalShortcuts[action]) return;
    if (conflictingShortcut(next)) { setSettingsError(copy.shortcutConflict); return; }
    void updateUiSettings({ terminalShortcuts: next });
  }

  async function changeSessionGrouping(sessionGrouping: SessionGrouping) {
    if (sessionGrouping === uiSettings.sessionGrouping) {
      return;
    }

    const previous = uiSettings;
    const next = { ...uiSettings, sessionGrouping };
    setUiSettings(next);
    setError("");
    try {
      await invoke("save_ui_settings", { uiSettings: next });
    } catch (reason) {
      setUiSettings(previous);
      setError(copy.settingsSaveError(describeError(reason)));
    }
  }

  async function toggleSessionSidebar() {
    if (!preferencesReady || sidebarSaving) return;
    const next = { ...uiSettings, sessionSidebarCollapsed: !uiSettings.sessionSidebarCollapsed };
    setSidebarSaving(true);
    try {
      await invoke("save_ui_settings", { uiSettings: next });
      setUiSettings(next);
      setError("");
    } catch (reason) {
      setError(copy.settingsSaveError(describeError(reason)));
    } finally {
      setSidebarSaving(false);
    }
  }

  async function sendInput(event?: FormEvent) {
    event?.preventDefault();
    if (
      !selectedSession ||
      selectedSession.status !== "running" ||
      inputSending ||
      (!input && attachments.length === 0)
    ) {
      return;
    }

    try {
      const payload = buildComposerPayload(input, attachments);
      const terminal = terminalHandlesRef.current.get(selectedSession.id)?.terminal;
      const terminalPayload = prepareTerminalSubmission(
        payload,
        terminal?.modes.bracketedPasteMode ?? false,
      );
      setInputSending(true);
      await invoke("write_terminal", {
        sessionId: selectedSession.id,
        data: terminalPayload,
      });
      await delay(SUBMIT_AFTER_INPUT_DELAY_MS);
      await invoke("write_terminal", {
        sessionId: selectedSession.id,
        data: "\r",
      });
      setInput("");
      setAttachments([]);
      terminalHandlesRef.current.get(selectedSession.id)?.terminal.focus();
    } catch (reason) {
      setError(describeError(reason));
    } finally {
      setInputSending(false);
    }
  }

  function handleInputKeyDown(event: ReactKeyboardEvent<HTMLTextAreaElement>) {
    if (event.key !== "Enter" || event.nativeEvent.isComposing) {
      return;
    }

    if (!event.shiftKey) {
      event.preventDefault();
      void sendInput();
      return;
    }

    const inputElement = event.currentTarget;
    if (inputElement.selectionStart !== inputElement.selectionEnd) {
      return;
    }

    const edit = buildNumberedLineBreak(input, inputElement.selectionStart);
    if (!edit) {
      return;
    }

    event.preventDefault();
    pendingInputCaretRef.current = edit.caret;
    setInput(edit.value);
  }

  async function attachFiles() {
    if (!isRunning) {
      return;
    }

    try {
      const selected = await openFileDialog({
        title: copy.attachDialogTitle,
        multiple: true,
        directory: false,
      });
      const selectedPaths = Array.isArray(selected) ? selected : selected ? [selected] : [];
      if (selectedPaths.length === 0) {
        return;
      }

      const next = Array.from(new Set([...attachments, ...selectedPaths]));
      if (next.length > MAX_ATTACHMENTS) {
        setError(copy.maxAttachments(MAX_ATTACHMENTS));
      }
      setAttachments(next.slice(0, MAX_ATTACHMENTS));
    } catch (reason) {
      setError(describeError(reason));
    }
  }

  function requestStopSelected() {
    if (!selectedSession || selectedSession.status !== "running") {
      return;
    }

    setError("");
    setStopConfirmation({
      kind: "selected",
      sessionId: selectedSession.id,
      sessionName: selectedSession.name,
    });
  }

  function requestStopAll() {
    if (activeCount === 0 || !activeWorkspace) {
      return;
    }

    setError("");
    setStopConfirmation({
      kind: "workspace",
      pageId: activeWorkspace.id,
      pageName: activeWorkspace.name,
      count: activeCount,
      closeAfterStop: false,
    });
  }

  async function confirmStop() {
    if (!stopConfirmation || stopStarting) {
      return;
    }

    try {
      setStopStarting(true);
      setError("");
      if (stopConfirmation.kind === "selected") {
        await invoke("stop_terminal", { sessionId: stopConfirmation.sessionId });
      } else {
        await invoke("stop_workspace_terminals", { pageId: stopConfirmation.pageId });
        if (stopConfirmation.closeAfterStop) {
          removeWorkspace(stopConfirmation.pageId);
        }
      }
      setStopConfirmation(null);
    } catch (reason) {
      setError(describeError(reason));
    } finally {
      setStopStarting(false);
    }
  }

  async function confirmApplicationExit() {
    if (shutdownStarting) {
      return;
    }

    try {
      setShutdownStarting(true);
      setError("");
      await invoke("shutdown_application");
    } catch (reason) {
      setError(describeError(reason));
      setShutdownStarting(false);
    }
  }

  function discardSessionResources(sessionId: number) {
    discardedSessionIdsRef.current.add(sessionId);
    knownSessionIdsRef.current.delete(sessionId);
    outputCacheRef.current.delete(sessionId);
    pendingExitRef.current.delete(sessionId);
    nextOutputOffsetRef.current.delete(sessionId);
    lastOutputAtRef.current.delete(sessionId);
    retainedPreviewsRef.current.delete(sessionId);
    const activityTimer = activityTimersRef.current.get(sessionId);
    if (activityTimer !== undefined) {
      window.clearTimeout(activityTimer);
      activityTimersRef.current.delete(sessionId);
    }
    const previewTimer = previewTimersRef.current.get(sessionId);
    if (previewTimer !== undefined) {
      window.clearTimeout(previewTimer);
      previewTimersRef.current.delete(sessionId);
    }
  }

  function removeSession(sessionId: number) {
    const session = sessions.find((candidate) => candidate.id === sessionId);
    if (!session || session.status === "running") {
      return;
    }

    const nextSessions = sessions.filter((candidate) => candidate.id !== sessionId);
    discardSessionResources(sessionId);
    setSessions(nextSessions);

    if (selectedSessionId === sessionId) {
      const remainingPageSessions = nextSessions.filter(
        (candidate) => candidate.pageId === activeWorkspaceId,
      );
      setSelectedSessionId(
        remainingPageSessions[remainingPageSessions.length - 1]?.id ?? null,
      );
    }
  }

  function getSessionSidebarMaxWidth() {
    const cardWidth = terminalCardRef.current?.getBoundingClientRect().width;
    if (!cardWidth) {
      return SESSION_SIDEBAR_MAX_WIDTH;
    }

    return Math.max(
      SESSION_SIDEBAR_MIN_WIDTH,
      Math.min(
        SESSION_SIDEBAR_MAX_WIDTH,
        cardWidth * SESSION_SIDEBAR_MAX_RATIO,
        cardWidth - TERMINAL_STAGE_MIN_WIDTH - PANEL_RESIZER_SIZE,
      ),
    );
  }

  function getInputPanelMaxHeight() {
    const stackHeight = workspaceStackRef.current?.getBoundingClientRect().height;
    if (!stackHeight) {
      return INPUT_PANEL_MAX_HEIGHT;
    }

    return Math.max(
      INPUT_PANEL_MIN_HEIGHT,
      Math.min(INPUT_PANEL_MAX_HEIGHT, stackHeight * INPUT_PANEL_MAX_RATIO),
    );
  }

  function beginPanelResize(
    event: ReactPointerEvent<HTMLDivElement>,
    cursor: "col-resize" | "row-resize",
    onMove: (event: PointerEvent) => void,
  ) {
    if (event.button !== 0) {
      return;
    }

    event.preventDefault();
    const previousCursor = document.body.style.cursor;
    const previousUserSelect = document.body.style.userSelect;
    document.body.style.cursor = cursor;
    document.body.style.userSelect = "none";
    setPanelResizing(true);

    const finish = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", finish);
      window.removeEventListener("pointercancel", finish);
      window.removeEventListener("blur", finish);
      document.body.style.cursor = previousCursor;
      document.body.style.userSelect = previousUserSelect;
      setPanelResizing(false);
    };

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", finish, { once: true });
    window.addEventListener("pointercancel", finish, { once: true });
    window.addEventListener("blur", finish, { once: true });
  }

  function beginSessionSidebarResize(event: ReactPointerEvent<HTMLDivElement>) {
    const card = terminalCardRef.current;
    if (!card) {
      return;
    }

    const cardLeft = card.getBoundingClientRect().left;
    beginPanelResize(event, "col-resize", (pointerEvent) => {
      setSessionSidebarWidth(
        clamp(
          pointerEvent.clientX - cardLeft,
          SESSION_SIDEBAR_MIN_WIDTH,
          getSessionSidebarMaxWidth(),
        ),
      );
    });
  }

  function resizeSessionSidebarFromKeyboard(event: ReactKeyboardEvent<HTMLDivElement>) {
    let nextWidth: number | null = null;
    if (event.key === "ArrowLeft") {
      nextWidth = sessionSidebarWidth - 16;
    } else if (event.key === "ArrowRight") {
      nextWidth = sessionSidebarWidth + 16;
    } else if (event.key === "Home") {
      nextWidth = SESSION_SIDEBAR_MIN_WIDTH;
    } else if (event.key === "End") {
      nextWidth = getSessionSidebarMaxWidth();
    }

    if (nextWidth === null) {
      return;
    }

    event.preventDefault();
    setSessionSidebarWidth(
      clamp(nextWidth, SESSION_SIDEBAR_MIN_WIDTH, getSessionSidebarMaxWidth()),
    );
  }

  function beginInputPanelResize(event: ReactPointerEvent<HTMLDivElement>) {
    const inputPanel = inputPanelRef.current;
    if (!inputPanel) {
      return;
    }

    const startPointerY = event.clientY;
    const startHeight = inputPanel.getBoundingClientRect().height;
    beginPanelResize(event, "row-resize", (pointerEvent) => {
      setInputPanelHeight(
        clamp(
          startHeight + startPointerY - pointerEvent.clientY,
          INPUT_PANEL_MIN_HEIGHT,
          getInputPanelMaxHeight(),
        ),
      );
    });
  }

  function resizeInputPanelFromKeyboard(event: ReactKeyboardEvent<HTMLDivElement>) {
    let nextHeight: number | null = null;
    if (event.key === "ArrowUp") {
      nextHeight = inputPanelHeight + 16;
    } else if (event.key === "ArrowDown") {
      nextHeight = inputPanelHeight - 16;
    } else if (event.key === "Home") {
      nextHeight = INPUT_PANEL_MIN_HEIGHT;
    } else if (event.key === "End") {
      nextHeight = getInputPanelMaxHeight();
    }

    if (nextHeight === null) {
      return;
    }

    event.preventDefault();
    setInputPanelHeight(
      clamp(nextHeight, INPUT_PANEL_MIN_HEIGHT, getInputPanelMaxHeight()),
    );
  }

  return (
    <main className="app-shell">
      <header className="app-header">
        <div>
          <h1 className="app-brand" aria-label="CLI Hub">
            <img src={cliHubIcon} alt="" width="32" height="32" />
            <span className="app-brand-wordmark">
              <span>CLI</span><span className="app-brand-accent">Hub</span>
            </span>
          </h1>
          {error && !launcherOpen && <p className="header-error">{error}</p>}
        </div>
        <div className="header-actions">
          <div className={`status status-${selectedStatus}`}>
            <span className="status-dot" aria-hidden="true" />
            {copy.status[selectedStatus]}
            <span className="process-id">
              {copy.active(activeCount, MAX_ACTIVE_SESSIONS_PER_WORKSPACE)}
            </span>
            <span className="process-id">
              {copy.totalActive(totalActiveCount, MAX_TOTAL_ACTIVE_SESSIONS)}
            </span>
            {selectedSession?.processId != null && (
              <span className="process-id">PID {selectedSession.processId}</span>
            )}
          </div>
          <button
            className="settings-button"
            type="button"
            aria-label={copy.settingsButton}
            title={copy.settingsButton}
            onClick={() => void openSettings()}
          >
            <svg aria-hidden="true" viewBox="0 0 24 24">
              <path d="M12 8.25a3.75 3.75 0 1 0 0 7.5 3.75 3.75 0 0 0 0-7.5Z" />
              <path d="m19.4 13.35 1.1.85-1.5 2.6-1.3-.5a7.8 7.8 0 0 1-1.7 1l-.2 1.4h-3l-.2-1.4a7.8 7.8 0 0 1-1.7-1l-1.3.5-1.5-2.6 1.1-.85a7.5 7.5 0 0 1 0-1.95l-1.1-.85 1.5-2.6 1.3.5a7.8 7.8 0 0 1 1.7-1l.2-1.4h3l.2 1.4a7.8 7.8 0 0 1 1.7 1l1.3-.5 1.5 2.6-1.1.85a7.5 7.5 0 0 1 0 1.95Z" />
            </svg>
          </button>
          <button
            className="new-session-button"
            type="button"
            onClick={openLauncher}
            disabled={launchDisabled}
            title={
              activeCount >= MAX_ACTIVE_SESSIONS_PER_WORKSPACE
                ? copy.maxSessions(MAX_ACTIVE_SESSIONS_PER_WORKSPACE)
                : totalActiveCount >= MAX_TOTAL_ACTIVE_SESSIONS
                  ? copy.globalMaxSessions(MAX_TOTAL_ACTIVE_SESSIONS)
                  : undefined
            }
          >
            <span aria-hidden="true">＋</span>
            {copy.newSession}
          </button>
        </div>
      </header>

      <div
        className="workspace-stack"
        ref={workspaceStackRef}
        style={
          {
            "--input-panel-height": `clamp(${INPUT_PANEL_MIN_HEIGHT}px, ${inputPanelHeight}px, min(${INPUT_PANEL_MAX_HEIGHT}px, ${INPUT_PANEL_MAX_RATIO * 100}%))`,
          } as CSSProperties
        }
      >
      <section
        className={`terminal-card ${uiSettings.sessionSidebarCollapsed ? "terminal-card-sidebar-collapsed" : ""}`}
        ref={terminalCardRef}
        aria-label={copy.terminalSessions}
        style={
          {
            "--session-sidebar-width": `${sessionSidebarWidth}px`,
          } as CSSProperties
        }
      >
        <div className="workspace-tabs-shell">
          <div className="workspace-tabs" role="tablist" aria-label={copy.workspaces}>
            {workspacePages.map((workspace) => {
              const workspaceActiveCount = sessions.filter(
                (session) =>
                  session.pageId === workspace.id && session.status === "running",
              ).length;
              const isActive = workspace.id === activeWorkspaceId;
              return (
                <div
                  className={`workspace-tab ${isActive ? "workspace-tab-active" : ""}`}
                  key={workspace.id}
                >
                  {editingWorkspaceId === workspace.id ? (
                    <input
                      className="workspace-tab-input"
                      value={workspaceNameDraft}
                      onChange={(event) => setWorkspaceNameDraft(event.currentTarget.value)}
                      onBlur={commitWorkspaceRename}
                      onKeyDown={(event) => {
                        if (event.key === "Enter") {
                          event.currentTarget.blur();
                        } else if (event.key === "Escape") {
                          setEditingWorkspaceId(null);
                        }
                      }}
                      maxLength={40}
                      autoFocus
                      aria-label={copy.renameWorkspace}
                    />
                  ) : (
                    <button
                      className="workspace-tab-main"
                      type="button"
                      role="tab"
                      aria-selected={isActive}
                      onClick={() => selectWorkspace(workspace.id)}
                      onDoubleClick={() => beginWorkspaceRename(workspace)}
                      title={copy.renameWorkspaceHint}
                    >
                      <span className="workspace-tab-content">
                        <span className="workspace-tab-name">{workspace.name}</span>
                        <small>
                          {workspaceActiveCount}/{MAX_ACTIVE_SESSIONS_PER_WORKSPACE}
                        </small>
                      </span>
                    </button>
                  )}
                  {workspacePages.length > 1 && (
                    <button
                      className="workspace-tab-close"
                      type="button"
                      aria-label={copy.closeWorkspace(workspace.name)}
                      title={copy.closeWorkspace(workspace.name)}
                      onClick={() => requestCloseWorkspace(workspace.id)}
                    >
                      ×
                    </button>
                  )}
                </div>
              );
            })}
            <button
              className="workspace-add-button"
              type="button"
              onClick={addWorkspace}
              disabled={workspacePages.length >= MAX_WORKSPACE_PAGES}
              aria-label={copy.addWorkspace}
              title={copy.addWorkspace}
            >
              ＋
            </button>
          </div>
          <span className="workspace-total-count">
            {copy.totalActive(totalActiveCount, MAX_TOTAL_ACTIVE_SESSIONS)}
          </span>
        </div>
        <div className="terminal-toolbar">
          <span>
            {selectedSession
              ? `${selectedSession.name} · ${copy.session(selectedSession.displayId ?? selectedSession.id)}`
              : copy.noSessionSelected}
          </span>
          <span>
            {isRunning
              ? copy.directKeyboard
              : copy.startOrSelect}
          </span>
        </div>

        <div className="workspace-sidebar">
          <div className="session-sidebar-header">
            <strong>{copy.sessionSidebarTitle}</strong>
            <button className="session-sidebar-toggle" type="button"
              aria-expanded={!uiSettings.sessionSidebarCollapsed}
              aria-controls="session-sidebar-overview session-sidebar-compact"
              aria-label={uiSettings.sessionSidebarCollapsed ? copy.expandSessionSidebar : copy.collapseSessionSidebar}
              title={uiSettings.sessionSidebarCollapsed ? copy.expandSessionSidebar : copy.collapseSessionSidebar}
              disabled={!preferencesReady || sidebarSaving}
              onClick={() => void toggleSessionSidebar()}>
              <span aria-hidden="true">{uiSettings.sessionSidebarCollapsed ? "»" : "«"}</span>
            </button>
          </div>
          <div className="session-icon-list" id="session-sidebar-compact" role="group"
            aria-label={copy.sessionOverview} ref={sidebarCompactRef}
            hidden={!uiSettings.sessionSidebarCollapsed}
            onScroll={(event) => {
              if (uiSettings.sessionSidebarCollapsed) {
                sidebarScrollPositionsRef.current.set(`${activeWorkspaceId}:compact`, event.currentTarget.scrollTop);
              }
            }}>
            {sessionGroups.flatMap((group) => group.sessions).map((session) => (
              <button className={`session-icon-button ${session.id === selectedSessionId ? "is-active" : ""}`}
                type="button" key={session.id} aria-pressed={session.id === selectedSessionId}
                aria-label={`${session.name} · ${copy.session(session.displayId ?? session.id)} · ${copy.status[session.status]}`}
                title={`${session.name} · ${copy.session(session.displayId ?? session.id)}\n${presetDefaults[session.preset].name} · ${copy.status[session.status]}\n${session.workingDirectory}`}
                onClick={() => setSelectedSessionId(session.id)}>
                <span className="session-icon-glyph" aria-hidden="true">
                  {({ powerShell: "PS", windowsPowerShell: "WP", cmd: ">_", codex: "Cx", claude: "Cl", custom: "CLI" } as const)[session.preset]}
                </span>
                <span className={`session-dot session-dot-${session.status}`} aria-hidden="true" />
                <small aria-hidden="true">{session.displayId ?? session.id}</small>
              </button>
            ))}
          </div>
          <div
            className="session-grouping-control"
            hidden={uiSettings.sessionSidebarCollapsed}
            role="group"
            aria-label={copy.groupingLabel}
          >
            {([
              ["directory", copy.groupByDirectory],
              ["cli", copy.groupByCli],
              ["none", copy.groupNone],
            ] as const).map(([value, label]) => (
              <button
                className={uiSettings.sessionGrouping === value ? "is-active" : ""}
                type="button"
                aria-pressed={uiSettings.sessionGrouping === value}
                title={`${copy.groupingLabel}: ${label}`}
                onClick={() => void changeSessionGrouping(value)}
                key={value}
              >
                {uiSettings.sessionGrouping === value && <span aria-hidden="true">✓ </span>}{label}
              </button>
            ))}
          </div>
          <div className="workspace-overview" id="session-sidebar-overview" aria-label={copy.sessionOverview}
            ref={sidebarOverviewRef} hidden={uiSettings.sessionSidebarCollapsed}
            onScroll={(event) => {
              if (!uiSettings.sessionSidebarCollapsed) {
                sidebarScrollPositionsRef.current.set(`${activeWorkspaceId}:overview`, event.currentTarget.scrollTop);
              }
            }}>
            {workspaceSessions.length === 0 && (
              <div className="workspace-overview-empty">{copy.emptyOverview}</div>
            )}
            {sessionGroups.map((group) => (
              <section className="workspace-group" key={group.key}>
                {group.showHeader && (
                  <div className="workspace-group-header" title={group.title}>
                    <strong>{group.label}</strong>
                    <span>{copy.sessionCount(group.sessions.length)}</span>
                  </div>
                )}
                <div className="session-card-grid">
                  {group.sessions.map((session) => (
                    <article
                      className={`session-card session-card-${session.status} ${
                        session.id === selectedSessionId ? "session-card-selected" : ""
                      } ${session.status !== "running" ? "session-card-dismissible" : ""}`}
                      key={session.id}
                    >
                      {session.status !== "running" && (
                        <button
                          className="session-card-remove"
                          type="button"
                          aria-label={copy.removeSession(session.name)}
                          title={copy.removeSessionPanel}
                          onClick={() => removeSession(session.id)}
                        >
                          <span aria-hidden="true">×</span>
                        </button>
                      )}
                      <button
                        className="session-card-main"
                        type="button"
                        aria-pressed={session.id === selectedSessionId}
                        onClick={() => setSelectedSessionId(session.id)}
                      >
                        <span className="session-card-heading">
                          <span>{session.name}</span>
                          <span className="session-card-status">
                            <span
                              className={`session-dot session-dot-${session.status}`}
                              aria-hidden="true"
                            />
                            {copy.status[session.status]}
                          </span>
                        </span>
                        <span className="session-preview">{session.historical ? copy.savedSessionRecord : session.preview}</span>
                      </button>
                      <footer className="session-card-footer">
                        <span>
                          {copy.session(session.displayId ?? session.id)}
                          {session.recovered ? ` · ${copy.recovered}` : ""}
                        </span>
                      </footer>
                    </article>
                  ))}
                </div>
              </section>
            ))}
          </div>
          <div className="workspace-sidebar-footer" hidden={uiSettings.sessionSidebarCollapsed}>
            <button
              className="danger-button session-stop-button"
              type="button"
              onClick={requestStopSelected}
              disabled={!isRunning}
            >
              {copy.stopSelected}
            </button>
            <button
              className="danger-button session-stop-button"
              type="button"
              onClick={requestStopAll}
              disabled={activeCount === 0}
            >
              {copy.stopAll}
            </button>
          </div>
        </div>

        <div
          className="panel-resizer session-sidebar-resizer"
          hidden={uiSettings.sessionSidebarCollapsed}
          role="separator"
          tabIndex={0}
          aria-label={copy.resizeSessionSidebar}
          aria-orientation="vertical"
          aria-valuemin={SESSION_SIDEBAR_MIN_WIDTH}
          aria-valuemax={SESSION_SIDEBAR_MAX_WIDTH}
          aria-valuenow={Math.round(sessionSidebarWidth)}
          title={copy.resizeSessionSidebarHint}
          onPointerDown={beginSessionSidebarResize}
          onKeyDown={resizeSessionSidebarFromKeyboard}
          onDoubleClick={() =>
            setSessionSidebarWidth(
              clamp(
                SESSION_SIDEBAR_DEFAULT_WIDTH,
                SESSION_SIDEBAR_MIN_WIDTH,
                getSessionSidebarMaxWidth(),
              ),
            )
          }
        />

        <div
          className={`terminal-stage ${terminalFocusMode ? "terminal-stage-focus" : ""}`}
          role="region"
          aria-label={copy.terminalStage}
          onDoubleClick={(event) => {
            if (!selectedSession) {
              return;
            }
            event.preventDefault();
            terminalHandlesRef.current.get(selectedSession.id)?.terminal.clearSelection();
            setTerminalFocusMode((current) => !current);
          }}
        >
          {terminalFocusMode && selectedSession && (
            <button
              className="terminal-focus-exit"
              type="button"
              aria-label={copy.exitTerminalFocus}
              title={copy.exitTerminalFocusHint}
              onClick={() => setTerminalFocusMode(false)}
              onDoubleClick={(event) => event.stopPropagation()}
            >
              <span aria-hidden="true">↙</span>
              {copy.exitTerminalFocus}
            </button>
          )}
          {workspaceSessions.length === 0 && (
            <div className="terminal-empty-state">
              <strong>{copy.readyTitle}</strong>
              <span>{copy.readyDetail}</span>
            </div>
          )}
          {workspaceSessions.map((session) => (
            <TerminalPane
              key={session.id}
              session={session}
              active={session.id === selectedSessionId}
              focusMode={terminalFocusMode && session.id === selectedSessionId}
              shortcuts={uiSettings.terminalShortcuts}
              onReady={registerTerminal}
              deferResize={panelResizing}
              onDispose={unregisterTerminal}
              onError={reportTerminalError}
            />
          ))}
        </div>
      </section>

      <div
        className="panel-resizer input-panel-resizer"
        role="separator"
        tabIndex={0}
        aria-label={copy.resizeInputPanel}
        aria-orientation="horizontal"
        aria-valuemin={INPUT_PANEL_MIN_HEIGHT}
        aria-valuemax={INPUT_PANEL_MAX_HEIGHT}
        aria-valuenow={Math.round(inputPanelHeight)}
        title={copy.resizeInputPanelHint}
        onPointerDown={beginInputPanelResize}
        onKeyDown={resizeInputPanelFromKeyboard}
        onDoubleClick={() =>
          setInputPanelHeight(
            clamp(
              INPUT_PANEL_DEFAULT_HEIGHT,
              INPUT_PANEL_MIN_HEIGHT,
              getInputPanelMaxHeight(),
            ),
          )
        }
      />

      <form
        className="input-panel"
        ref={inputPanelRef}
        onSubmit={(event) => void sendInput(event)}
      >
        <label htmlFor="terminal-input">{copy.inputLabel}</label>
        <div className="input-row">
          <div
            className={`composer-box ${!isRunning || inputSending ? "composer-box-disabled" : ""}`}
          >
            <textarea
              id="terminal-input"
              ref={inputRef}
              rows={1}
              value={input}
              onChange={(event) => setInput(event.currentTarget.value)}
              onKeyDown={handleInputKeyDown}
              placeholder={
                isRunning ? copy.sendTo(selectedSession.name) : copy.selectRunning
              }
              disabled={!isRunning || inputSending}
              autoComplete="off"
              spellCheck={false}
            />
            {attachments.length > 0 && (
              <div className="attachment-list" aria-label={copy.attachmentList}>
                {attachments.map((path) => (
                  <span className="attachment-chip" key={path} title={path}>
                    <span>{fileNameFromPath(path)}</span>
                    <button
                      type="button"
                      aria-label={copy.removeAttachment(fileNameFromPath(path))}
                      onClick={() =>
                        setAttachments((current) => current.filter((candidate) => candidate !== path))
                      }
                    >
                      ×
                    </button>
                  </span>
                ))}
              </div>
            )}
            <div className="composer-footer">
              <button
                className="attachment-button"
                type="button"
                onClick={() => void attachFiles()}
                disabled={!isRunning || inputSending}
              >
                <span aria-hidden="true">＋</span>
                {copy.attach}
              </button>
              <span>{copy.composerHint}</span>
            </div>
          </div>
          <button
            className="primary-button send-button"
            type="submit"
            disabled={
              !isRunning || inputSending || (!input && attachments.length === 0)
            }
          >
            {copy.send}
          </button>
        </div>
        <SessionStatusBar session={selectedSession} copy={copy} />
      </form>
      </div>

      {launcherOpen && (
        <div className="modal-backdrop">
          <section
            className="launcher-dialog new-session-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="launcher-title"
          >
            <header className="launcher-dialog-header">
              <div>
                <p className="eyebrow">{copy.launchEyebrow}</p>
                <h2 id="launcher-title">{copy.launchTitle}</h2>
                <p>{copy.launchDescription}</p>
              </div>
              <button
                className="dialog-close"
                type="button"
                aria-label={copy.closeLaunch}
                onClick={closeLauncher}
                disabled={launchStarting}
              >
                ×
              </button>
            </header>

            <form
              className="launcher-form"
              onSubmit={(event) => {
                event.preventDefault();
                void startTerminal();
              }}
            >
              <div className="profile-panel">
                <div className="field profile-select-field">
                  <label htmlFor="saved-profile">{copy.savedProfile}</label>
                  <select
                    id="saved-profile"
                    value={selectedProfileId}
                    onChange={(event) => selectLaunchProfile(event.currentTarget.value)}
                    disabled={launchStarting}
                  >
                    <option value="">{copy.unsavedConfiguration}</option>
                    {launchProfiles.map((profile) => (
                      <option value={profile.id} key={profile.id}>
                        {profile.label}
                      </option>
                    ))}
                  </select>
                </div>
                <div className="profile-actions">
                  <button
                    type="button"
                    onClick={saveLaunchProfile}
                    disabled={launchStarting || profileSaving || !preferencesReady}
                  >
                    {selectedProfileId ? copy.updateProfile : copy.saveProfile}
                  </button>
                  <button
                    type="button"
                    onClick={deleteLaunchProfile}
                    disabled={
                      launchStarting || profileSaving || !preferencesReady || !selectedProfileId
                    }
                  >
                    {copy.delete}
                  </button>
                </div>
                <p>{copy.environmentNotSaved}</p>
              </div>

              <div className="launch-grid">
                <div className="field">
                  <label htmlFor="launch-preset">{copy.cliPreset}</label>
                  <select
                    id="launch-preset"
                    value={agentTemplate ?? preset}
                    onChange={(event) => changePreset(event.currentTarget.value as LaunchPreset | AgentTemplate)}
                    disabled={launchStarting}
                    autoFocus
                  >
                    <option value="powerShell">PowerShell ({copy.recommended})</option>
                    <option value="windowsPowerShell">Windows PowerShell</option>
                    <option value="cmd">Command Prompt</option>
                    <option value="codex">Codex</option>
                    <option value="claude">Claude Code</option>
                    <option value="pi">Pi</option>
                    <option value="hermes">Hermes Agent</option>
                    <option value="opencode">OpenCode</option>
                    <option value="custom">{copy.customCli}</option>
                  </select>
                </div>

                <div className="field">
                  <label htmlFor="session-name">{copy.displayName}</label>
                  <input
                    id="session-name"
                    value={sessionName}
                    onChange={(event) => setSessionName(event.currentTarget.value)}
                    disabled={launchStarting}
                    spellCheck={false}
                  />
                </div>

                <div className="field field-wide">
                  <label htmlFor="executable">{copy.executable}</label>
                  <input
                    id="executable"
                    value={executable}
                    onChange={(event) => {
                      setExecutable(event.currentTarget.value);
                      setAgentTemplate(null);
                    }}
                    placeholder="C:\\Tools\\agent.exe"
                    disabled={launchStarting || preset !== "custom"}
                    spellCheck={false}
                  />
                </div>

                <div className="field">
                  <label htmlFor="arguments">{copy.arguments}</label>
                  <textarea
                    id="arguments"
                    value={argumentsText}
                    onChange={(event) => setArgumentsText(event.currentTarget.value)}
                    placeholder={
                      preset === "codex" || preset === "claude"
                        ? "--version"
                        : "--flag\nvalue with spaces"
                    }
                    disabled={launchStarting}
                    spellCheck={false}
                  />
                </div>

                <div className="field">
                  <label htmlFor="environment">{copy.environment}</label>
                  <textarea
                    id="environment"
                    value={environmentText}
                    onChange={(event) => setEnvironmentText(event.currentTarget.value)}
                    placeholder="NO_COLOR=1"
                    disabled={launchStarting}
                    spellCheck={false}
                  />
                </div>

                <div className="field field-wide">
                  <label htmlFor="working-directory">{copy.workingDirectory}</label>
                  <input
                    id="working-directory"
                    list="recent-working-directories"
                    value={workingDirectory}
                    onChange={(event) => setWorkingDirectory(event.currentTarget.value)}
                    placeholder="D:\\Demo"
                    disabled={launchStarting}
                    spellCheck={false}
                  />
                  <datalist id="recent-working-directories">
                    {recentDirectories.map((directory) => (
                      <option value={directory} key={directory} />
                    ))}
                  </datalist>
                </div>
              </div>

              {error && <p className="error-message">{error}</p>}

              <footer className="launcher-actions">
                <button type="button" onClick={closeLauncher} disabled={launchStarting}>
                  {copy.cancel}
                </button>
                <button className="primary-button" type="submit" disabled={launchDisabled}>
                  {launchStarting ? copy.starting : copy.start(sessionName.trim() || "CLI")}
                </button>
              </footer>
            </form>
          </section>
        </div>
      )}

      {settingsOpen && (
        <div className="modal-backdrop settings-backdrop">
          <section
            className="launcher-dialog settings-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="settings-title"
          >
            <header className="launcher-dialog-header">
              <div>
                <p className="eyebrow">{copy.settingsEyebrow}</p>
                <h2 id="settings-title">{copy.settingsTitle}</h2>
                <p>{copy.settingsDescription}</p>
              </div>
              <button
                className="dialog-close"
                type="button"
                aria-label={copy.closeSettings}
                onClick={closeSettings}
                disabled={autostartLoading || updateActivity === "downloading"}
              >
                ×
              </button>
            </header>

            <div className="settings-body">
              <section className="settings-section background-settings" aria-labelledby="background-setting-title">
                <strong className="settings-section-title" id="background-setting-title">{copy.backgroundTitle}</strong>
                <p className="settings-section-description">{copy.backgroundDescription}</p>
                <div className="settings-section-actions">
                  <button className="settings-action-button" type="button"
                    disabled={shutdownStarting || updateActivity === "downloading"}
                    onClick={() => { setSettingsOpen(false); setCloseRequestCount(totalActiveCount); }}>
                    {copy.stopAllAndExit}
                  </button>
                </div>
              </section>
              <section className="settings-section" aria-labelledby="startup-setting-title">
                <label className="settings-option" htmlFor="autostart-enabled">
                  <span>
                    <strong id="startup-setting-title">{copy.startupTitle}</strong>
                    <small>
                      {autostartLoading ? copy.checkingStartup : copy.startupDescription}
                    </small>
                  </span>
                  <input
                    id="autostart-enabled"
                    className="toggle-input"
                    type="checkbox"
                    checked={autostartEnabled}
                    onChange={(event) => void changeAutostart(event.currentTarget.checked)}
                    disabled={autostartLoading}
                  />
                </label>

                <label className="settings-option" htmlFor="start-minimized">
                  <span>
                    <strong>{copy.minimizedTitle}</strong>
                    <small>{copy.minimizedDescription}</small>
                  </span>
                  <input
                    id="start-minimized"
                    className="toggle-input"
                    type="checkbox"
                    checked={uiSettings.startMinimized}
                    onChange={(event) =>
                      updateUiSettings({ startMinimized: event.currentTarget.checked })
                    }
                    disabled={!autostartEnabled || autostartLoading}
                  />
                </label>
              </section>

              <section className="settings-section">
                <label className="settings-option">
                  <span><strong>{copy.restoreTitle}</strong><small>{copy.restoreDescription}</small></span>
                  <input className="toggle-input" type="checkbox" checked={uiSettings.restoreOnStartup}
                    onChange={(event) => void updateUiSettings({ restoreOnStartup: event.currentTarget.checked })} />
                </label>
              </section>
              <section className="settings-section shortcut-settings">
                <strong className="settings-section-title">{copy.shortcutsTitle}</strong>
                <p className="settings-section-description">{copy.shortcutsDescription}</p>
                {([ ["copy", copy.shortcutCopy], ["paste", copy.shortcutPaste],
                  ["pasteAlternate", copy.shortcutPasteAlternate], ["interrupt", copy.shortcutInterrupt],
                  ["newline", copy.shortcutNewline] ] as const).map(([action, label]) => (
                  <label className="shortcut-row" key={action}>
                    <span>{label}</span>
                    <input className="shortcut-key-input" aria-label={label} readOnly value={uiSettings.terminalShortcuts[action]}
                      onKeyDown={(event) => captureShortcut(action, event)} />
                  </label>
                ))}
                <label className="settings-option">
                  <span><strong>{copy.smartCopyTitle}</strong><small>{copy.smartCopyDescription}</small></span>
                  <input className="toggle-input" type="checkbox" checked={uiSettings.terminalShortcuts.smartCopy}
                    onChange={(event) => void updateUiSettings({ terminalShortcuts: { ...uiSettings.terminalShortcuts, smartCopy: event.currentTarget.checked } })} />
                </label>
                <div className="shortcut-settings-actions">
                  <button className="settings-action-button" type="button"
                    onClick={() => void updateUiSettings({ terminalShortcuts: DEFAULT_TERMINAL_SHORTCUTS })}>
                    {copy.shortcutReset}
                  </button>
                </div>
              </section>
              <section className="settings-section language-setting">
                <div>
                  <label htmlFor="interface-color-mode">{copy.colorModeTitle}</label>
                  <p>{copy.colorModeDescription}</p>
                </div>
                <select
                  id="interface-color-mode"
                  value={uiSettings.colorMode}
                  onChange={(event) =>
                    updateUiSettings({
                      colorMode: event.currentTarget.value as ColorModePreference,
                    })
                  }
                >
                  <option value="system">{copy.followSystem}</option>
                  <option value="light">{copy.lightMode}</option>
                  <option value="dark">{copy.darkMode}</option>
                </select>
              </section>

              <section className="settings-section language-setting">
                <div>
                  <label htmlFor="interface-language">{copy.languageTitle}</label>
                  <p>{copy.languageDescription}</p>
                </div>
                <select
                  id="interface-language"
                  value={uiSettings.language}
                  onChange={(event) =>
                    updateUiSettings({ language: event.currentTarget.value as LanguagePreference })
                  }
                >
                  <option value="system">{copy.followSystem}</option>
                  <option value="en">{copy.english}</option>
                  <option value="zh-CN">{copy.simplifiedChinese}</option>
                </select>
              </section>

              <section className="settings-section update-setting" aria-labelledby="update-title">
                <div>
                  <strong id="update-title">{copy.updateTitle}</strong>
                  <p>{copy.updateDescription}</p>
                  <small>{copy.currentVersion(appVersion || "…")}</small>
                  {updateActivity === "available" && availableUpdate && (
                    <small className="update-status update-status-available">
                      {copy.updateAvailable(availableUpdate.version)}
                    </small>
                  )}
                  {updateActivity === "latest" && (
                    <small className="update-status">{copy.upToDate}</small>
                  )}
                  {updateActivity === "downloading" && (
                    <small className="update-status update-status-available">
                      {copy.downloadingUpdate(updateProgress)}
                    </small>
                  )}
                  {updateActivity === "available" && totalActiveCount > 0 && (
                    <small className="update-status update-status-warning">
                      {copy.updateRequiresIdle}
                    </small>
                  )}
                </div>
                <button
                  className="settings-action-button settings-update-button"
                  type="button"
                  onClick={() =>
                    void (availableUpdate ? installAvailableUpdate() : checkForUpdates())
                  }
                  disabled={
                    updateActivity === "checking" ||
                    updateActivity === "downloading" ||
                    !listenersReady
                  }
                >
                  {updateActivity === "checking"
                    ? copy.checkingForUpdates
                    : updateActivity === "downloading"
                      ? copy.installingUpdate
                      : availableUpdate
                        ? copy.installUpdate(availableUpdate.version)
                        : copy.checkForUpdates}
                </button>
              </section>

              {settingsError && <p className="error-message">{settingsError}</p>}

              <footer className="launcher-actions settings-actions">
                <button
                  className="primary-button"
                  type="button"
                  onClick={closeSettings}
                  disabled={autostartLoading || updateActivity === "downloading"}
                >
                  {copy.done}
                </button>
              </footer>
            </div>
          </section>
        </div>
      )}

      {stopConfirmation && (
        <div className="modal-backdrop stop-confirmation-backdrop">
          <section
            className="launcher-dialog shutdown-dialog stop-confirmation-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="stop-confirmation-title"
          >
            <header className="launcher-dialog-header">
              <div>
                <p className="eyebrow">{copy.stopWarningEyebrow}</p>
                <h2 id="stop-confirmation-title">
                  {stopConfirmation.kind === "selected"
                    ? copy.stopSelectedTitle(stopConfirmation.sessionName)
                    : stopConfirmation.closeAfterStop
                      ? copy.closeWorkspaceTitle(stopConfirmation.pageName)
                      : copy.stopWorkspaceTitle(
                          stopConfirmation.pageName,
                          stopConfirmation.count,
                        )}
                </h2>
                <p>
                  {stopConfirmation.kind === "selected"
                    ? copy.stopSelectedMessage
                    : stopConfirmation.closeAfterStop
                      ? copy.closeWorkspaceMessage
                      : copy.stopWorkspaceMessage}
                </p>
              </div>
            </header>
            <div className="shutdown-dialog-body">
              <p>{copy.stopWarningDetail}</p>
              {error && <p className="error-message">{error}</p>}
              <footer className="launcher-actions stop-confirmation-actions">
                <button
                  type="button"
                  onClick={() => {
                    setStopConfirmation(null);
                    setError("");
                  }}
                  disabled={stopStarting}
                  autoFocus
                >
                  {copy.cancel}
                </button>
                <button
                  className="danger-button shutdown-confirm-button"
                  type="button"
                  onClick={() => void confirmStop()}
                  disabled={stopStarting}
                >
                  {stopStarting
                    ? copy.stopping
                    : stopConfirmation.kind === "selected"
                      ? copy.confirmStopSelected
                      : stopConfirmation.closeAfterStop
                        ? copy.stopAndCloseWorkspace
                        : copy.confirmStopWorkspace}
                </button>
              </footer>
            </div>
          </section>
        </div>
      )}

      {closeRequestCount !== null && (
        <div className="modal-backdrop shutdown-backdrop">
          <section
            className="launcher-dialog shutdown-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="shutdown-title"
          >
            <header className="launcher-dialog-header">
              <div>
                <p className="eyebrow">{copy.shutdownEyebrow}</p>
                <h2 id="shutdown-title">{copy.shutdownTitle}</h2>
                <p>{copy.shutdownMessage(closeRequestCount)}</p>
              </div>
            </header>
            <div className="shutdown-dialog-body">
              <p>{copy.shutdownDetail}</p>
              {error && <p className="error-message">{error}</p>}
              <footer className="launcher-actions">
                <button
                  type="button"
                  onClick={() => setCloseRequestCount(null)}
                  disabled={shutdownStarting}
                  autoFocus
                >
                  {copy.cancel}
                </button>
                <button
                  className="danger-button shutdown-confirm-button"
                  type="button"
                  onClick={() => void confirmApplicationExit()}
                  disabled={shutdownStarting}
                >
                  {shutdownStarting ? copy.stoppingSessions : copy.stopAllAndExit}
                </button>
              </footer>
            </div>
          </section>
        </div>
      )}
    </main>
  );
}

type UiCopy = (typeof UI_COPY)[keyof typeof UI_COPY];

interface SessionStatusBarProps {
  session: SessionRecord | null;
  copy: UiCopy;
}

function SessionStatusBar({ session, copy }: SessionStatusBarProps) {
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    setNow(Date.now());
    if (session?.lastOutputAt == null) {
      return;
    }

    const timer = window.setInterval(() => setNow(Date.now()), 1_000);
    return () => window.clearInterval(timer);
  }, [session?.id, session?.lastOutputAt]);

  if (!session) {
    return (
      <div className="session-status-bar session-status-bar-idle" aria-label={copy.sessionStatusBar}>
        <span className="session-status-primary">
          <span className="session-status-indicator" aria-hidden="true" />
          {copy.noSessionSelected}
        </span>
        <span className="session-status-empty-hint">{copy.startOrSelect}</span>
      </div>
    );
  }

  const elapsed =
    session.lastOutputAt == null ? null : Math.max(0, now - session.lastOutputAt);
  const quiet = session.status === "running" && elapsed != null && elapsed >= QUIET_AFTER_MS;
  const statusLabel = quiet ? copy.quiet : copy.status[session.status];
  const activityLabel = formatSessionActivity(copy, session, elapsed);

  return (
    <div
      className={`session-status-bar session-status-bar-${quiet ? "quiet" : session.status}`}
      aria-label={copy.sessionStatusBar}
    >
      <span className="session-status-primary">
        <span className="session-status-indicator" aria-hidden="true" />
        {statusLabel}
      </span>
      <span className="session-status-name" title={`${session.name} · ${copy.session(session.displayId ?? session.id)}`}>
        {session.name} · {copy.session(session.displayId ?? session.id)}
      </span>
      <span className="session-status-path" title={session.workingDirectory}>
        {session.workingDirectory}
      </span>
      <span className="session-status-activity">{activityLabel}</span>
    </div>
  );
}

function formatSessionActivity(
  copy: UiCopy,
  session: SessionRecord,
  elapsed: number | null,
): string {
  if (elapsed == null) {
    if (session.recovered) {
      return copy.outputTimeUnavailable;
    }
    return session.status === "running" ? copy.waitingForOutput : copy.noOutputRecorded;
  }
  if (elapsed < 5_000) {
    return copy.outputJustNow;
  }
  if (elapsed < 60_000) {
    return copy.lastOutputSeconds(Math.floor(elapsed / 1_000));
  }
  if (elapsed < 3_600_000) {
    return copy.lastOutputMinutes(Math.floor(elapsed / 60_000));
  }
  return copy.lastOutputHours(Math.floor(elapsed / 3_600_000));
}

interface TerminalPaneProps {
  session: SessionRecord;
  active: boolean;
  focusMode: boolean;
  deferResize: boolean;
  shortcuts: TerminalShortcuts;
  onReady: (sessionId: number, terminal: Terminal, fitAddon: FitAddon) => void;
  onDispose: (sessionId: number, terminal: Terminal) => void;
  onError: (reason: unknown) => void;
}

function TerminalPane({
  session,
  active,
  focusMode,
  deferResize,
  shortcuts,
  onReady,
  onDispose,
  onError,
}: TerminalPaneProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const runningRef = useRef(session.status === "running");
  const shortcutsRef = useRef(shortcuts);
  shortcutsRef.current = shortcuts;

  useEffect(() => {
    runningRef.current = session.status === "running";
  }, [session.status]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) {
      return;
    }

    const terminal = new Terminal({
      convertEol: false,
      cursorBlink: true,
      cursorStyle: "bar",
      fontFamily: TERMINAL_FONT_FAMILY,
      fontSize: 14,
      scrollback: 5_000,
      theme: {
        background: "#101418",
        foreground: "#d8e0e8",
        cursor: "#7dd3fc",
        selectionBackground: "#28536b",
      },
    });
    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.open(container);
    terminalRef.current = terminal;
    fitAddonRef.current = fitAddon;
    onReady(session.id, terminal, fitAddon);

    terminal.attachCustomKeyEventHandler((event) => {
      const mapping = shortcutsRef.current;
      const newlineData = terminalNewlineData(event, session.preset, mapping.newline);
      if (newlineData !== null) {
        event.preventDefault();
        event.stopPropagation();
        if (event.type === "keydown" && runningRef.current) {
          void invoke("write_terminal", { sessionId: session.id, data: newlineData }).catch(onError);
        }
        return false;
      }
      const shortcut = shortcutFromEvent(event);
      const interrupt = shortcut === mapping.interrupt;
      const copySelection = shortcut === mapping.copy || (interrupt && mapping.smartCopy && terminal.hasSelection());
      const paste = shortcut === mapping.paste || shortcut === mapping.pasteAlternate;
      if (!copySelection && !paste && !interrupt) return true;
      event.preventDefault();
      event.stopPropagation();
      if (event.type !== "keydown" || event.repeat) return false;
      if (copySelection) {
        if (terminal.hasSelection()) void writeClipboardText(terminal.getSelection()).catch(onError);
      } else if (paste) {
        if (runningRef.current) void readClipboardText().then((text) => {
          if (runningRef.current && terminalRef.current === terminal) terminal.paste(text);
        }).catch(onError);
      } else if (runningRef.current) {
        void invoke("write_terminal", { sessionId: session.id, data: "\x03" }).catch(onError);
      }
      return false;
    });

    const inputDisposable = terminal.onData((data) => {
      if (runningRef.current) {
        void invoke("write_terminal", { sessionId: session.id, data }).catch(onError);
      }
    });

    return () => {
      inputDisposable.dispose();
      onDispose(session.id, terminal);
      terminal.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
    };
  }, [onDispose, onError, onReady, session.id]);

  useEffect(() => {
    // FitAddon clears/reflows the screen on size changes. Keep the existing
    // terminal mounted while dragging and fit once when the pointer is released.
    if (!active || deferResize) {
      return;
    }

    let resizeFrame: number | null = null;
    const resize = (focusTerminal: boolean) => {
      if (resizeFrame !== null) {
        window.cancelAnimationFrame(resizeFrame);
      }
      resizeFrame = window.requestAnimationFrame(() => {
        resizeFrame = null;
        const terminal = terminalRef.current;
        const fitAddon = fitAddonRef.current;
        if (!terminal || !fitAddon) {
          return;
        }
        const previousCols = terminal.cols;
        const previousRows = terminal.rows;
        fitAddon.fit();
        if (focusTerminal) {
          terminal.focus();
        }
        if (session.status === "running" &&
            (focusTerminal || terminal.cols !== previousCols || terminal.rows !== previousRows)) {
          void invoke("resize_terminal", {
            sessionId: session.id,
            cols: terminal.cols,
            rows: terminal.rows,
          }).catch(() => undefined);
        }
      });
    };
    const handleWindowResize = () => resize(false);
    const resizeObserver = new ResizeObserver(() => resize(false));
    const container = containerRef.current;

    resize(true);
    if (container) {
      resizeObserver.observe(container);
    }
    window.addEventListener("resize", handleWindowResize);
    return () => {
      resizeObserver.disconnect();
      window.removeEventListener("resize", handleWindowResize);
      if (resizeFrame !== null) {
        window.cancelAnimationFrame(resizeFrame);
      }
    };
  }, [active, focusMode, deferResize, session.id, session.status]);

  return (
    <div
      className={`terminal-container ${active ? "terminal-container-active" : ""}`}
      ref={containerRef}
      role="tabpanel"
      aria-hidden={!active}
    />
  );
}

function groupSessions(
  sessions: SessionRecord[],
  grouping: SessionGrouping,
): SessionGroup[] {
  if (grouping === "none") {
    return sessions.length === 0
      ? []
      : [{ key: "all", label: "", showHeader: false, sessions }];
  }

  const groups = new Map<string, SessionGroup>();

  sessions.forEach((session) => {
    const key = grouping === "directory" ? session.workingDirectoryKey : session.preset;
    const existing = groups.get(key);
    if (existing) {
      existing.sessions.push(session);
    } else {
      const directoryGrouping = grouping === "directory";
      groups.set(key, {
        key,
        label: directoryGrouping
          ? directoryLabel(session.workingDirectory)
          : presetDefaults[session.preset].name,
        title: directoryGrouping ? session.workingDirectory : presetDefaults[session.preset].name,
        showHeader: true,
        sessions: [session],
      });
    }
  });

  return Array.from(groups.values());
}

function buildComposerPayload(input: string, attachments: string[]): string {
  if (attachments.length === 0) {
    return input;
  }

  const attachmentBlock = [
    "Attached local files (path references only):",
    ...attachments.map((path) => `- ${path}`),
  ].join("\n");
  return input ? `${input}\n\n${attachmentBlock}` : attachmentBlock;
}

function prepareTerminalSubmission(payload: string, bracketedPasteMode: boolean): string {
  if (!bracketedPasteMode) {
    return payload;
  }

  const normalizedPayload = payload.replace(/\r?\n/g, "\r");
  return `\u001b[200~${normalizedPayload}\u001b[201~`;
}

function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, milliseconds));
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(Math.max(value, minimum), maximum);
}

function buildNumberedLineBreak(
  value: string,
  caret: number,
): { value: string; caret: number } | null {
  const lineStart = value.lastIndexOf("\n", caret - 1) + 1;
  const line = value.slice(lineStart, caret);
  const match = /^(\s*)(\d+)([.)、])\s(.*)$/.exec(line);
  if (!match) {
    return null;
  }

  const [, indentation, numberText, delimiter, content] = match;
  if (!content.trim()) {
    const nextValue = `${value.slice(0, lineStart)}${indentation}${value.slice(caret)}`;
    return { value: nextValue, caret: lineStart + indentation.length };
  }

  const nextLine = `\n${indentation}${Number(numberText) + 1}${delimiter} `;
  const nextValue = `${value.slice(0, caret)}${nextLine}${value.slice(caret)}`;
  return { value: nextValue, caret: caret + nextLine.length };
}

function fileNameFromPath(path: string): string {
  const segments = path.split(/[\\/]/).filter(Boolean);
  return segments[segments.length - 1] ?? path;
}

function directoryLabel(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

function queuePendingOutput(
  pendingBySession: Map<number, PendingOutput>,
  output: TerminalOutput,
  alreadyTruncated = false,
) {
  let chunk = new Uint8Array(output.bytes);
  let offset = output.offset;
  const pending = pendingBySession.get(output.sessionId) ?? {
    chunks: [],
    byteLength: 0,
    truncated: false,
  };
  pending.truncated ||= alreadyTruncated;

  if (chunk.byteLength > MAX_PENDING_OUTPUT_BYTES) {
    offset += chunk.byteLength - MAX_PENDING_OUTPUT_BYTES;
    chunk = chunk.slice(chunk.byteLength - MAX_PENDING_OUTPUT_BYTES);
    pending.chunks = [];
    pending.byteLength = 0;
    pending.truncated = true;
  }

  pending.chunks.push({ offset, bytes: chunk });
  pending.chunks.sort((left, right) => left.offset - right.offset);
  pending.chunks = mergePendingChunks(pending.chunks);
  pending.byteLength = pending.chunks.reduce((total, item) => total + item.bytes.byteLength, 0);

  while (pending.byteLength > MAX_PENDING_OUTPUT_BYTES && pending.chunks.length > 0) {
    const excess = pending.byteLength - MAX_PENDING_OUTPUT_BYTES;
    const first = pending.chunks[0];
    if (first.bytes.byteLength <= excess) {
      pending.chunks.shift();
      pending.byteLength -= first.bytes.byteLength;
    } else {
      first.offset += excess;
      first.bytes = first.bytes.slice(excess);
      pending.byteLength -= excess;
    }
    pending.truncated = true;
  }

  pendingBySession.set(output.sessionId, pending);
}

function mergePendingChunks(
  chunks: Array<{ offset: number; bytes: Uint8Array }>,
): Array<{ offset: number; bytes: Uint8Array }> {
  const merged: Array<{ offset: number; bytes: Uint8Array }> = [];

  for (const chunk of chunks) {
    const previous = merged[merged.length - 1];
    if (!previous) {
      merged.push(chunk);
      continue;
    }

    const previousEnd = previous.offset + previous.bytes.byteLength;
    const chunkEnd = chunk.offset + chunk.bytes.byteLength;
    if (chunk.offset > previousEnd) {
      merged.push(chunk);
      continue;
    }
    if (chunkEnd <= previousEnd) {
      continue;
    }

    const tail = chunk.bytes.slice(previousEnd - chunk.offset);
    const combined = new Uint8Array(previous.bytes.byteLength + tail.byteLength);
    combined.set(previous.bytes);
    combined.set(tail, previous.bytes.byteLength);
    previous.bytes = combined;
  }

  return merged;
}

function takeUnseenOutput(
  nextOffsetBySession: Map<number, number>,
  output: TerminalOutput,
): { bytes: Uint8Array; gap: boolean } | null {
  const bytes = new Uint8Array(output.bytes);
  const endOffset = output.offset + bytes.byteLength;
  const expectedOffset = nextOffsetBySession.get(output.sessionId) ?? output.offset;

  if (endOffset <= expectedOffset) {
    return null;
  }

  const startIndex = Math.max(0, expectedOffset - output.offset);
  nextOffsetBySession.set(output.sessionId, endOffset);
  return {
    bytes: bytes.slice(startIndex),
    gap: output.offset > expectedOffset,
  };
}

function writeExitNotice(terminal: Terminal, exit: TerminalExit) {
  if (exit.error) {
    terminal.writeln(`\r\n[CLI Hub] Terminal failed: ${exit.error}`);
  } else {
    terminal.writeln(`\r\n[CLI Hub] Process exited with code ${exit.exitCode ?? "unknown"}.`);
  }
}

function parseArguments(value: string): string[] {
  return value.split(/\r?\n/).filter((argument) => argument.length > 0);
}

function parseEnvironment(value: string): EnvironmentVariable[] {
  return value
    .split(/\r?\n/)
    .filter((line) => line.trim().length > 0)
    .map((line, index) => {
      const separator = line.indexOf("=");
      if (separator <= 0) {
        throw new Error(`Environment line ${index + 1} must use NAME=VALUE`);
      }
      return {
        name: line.slice(0, separator).trim(),
        value: line.slice(separator + 1),
      };
    });
}

function describeError(reason: unknown): string {
  return reason instanceof Error ? reason.message : String(reason);
}

function readLaunchProfiles(): LaunchProfile[] {
  try {
    const value: unknown = JSON.parse(localStorage.getItem(LAUNCH_PROFILES_STORAGE_KEY) ?? "[]");
    if (!Array.isArray(value)) {
      return [];
    }
    return value.filter(isLaunchProfile).slice(0, MAX_LAUNCH_PROFILES);
  } catch {
    return [];
  }
}

function readRecentDirectories(): string[] {
  try {
    const value: unknown = JSON.parse(
      localStorage.getItem(RECENT_DIRECTORIES_STORAGE_KEY) ?? "[]",
    );
    if (!Array.isArray(value)) {
      return [];
    }
    return value
      .filter((directory): directory is string => typeof directory === "string")
      .slice(0, MAX_RECENT_DIRECTORIES);
  } catch {
    return [];
  }
}

function readUiSettings(): UiSettings {
  try {
    const value: unknown = JSON.parse(
      localStorage.getItem(UI_SETTINGS_STORAGE_KEY) ?? "null",
    );
    if (!value || typeof value !== "object") {
      return DEFAULT_UI_SETTINGS;
    }

    const settings = value as Record<string, unknown>;
    const language = isLanguagePreference(settings.language)
      ? settings.language
      : DEFAULT_UI_SETTINGS.language;
    const colorMode = isColorModePreference(settings.colorMode)
      ? settings.colorMode
      : DEFAULT_UI_SETTINGS.colorMode;
    const startMinimized =
      typeof settings.startMinimized === "boolean"
        ? settings.startMinimized
        : DEFAULT_UI_SETTINGS.startMinimized;
    const sessionGrouping = isSessionGrouping(settings.sessionGrouping)
      ? settings.sessionGrouping
      : DEFAULT_UI_SETTINGS.sessionGrouping;
    return { language, colorMode, startMinimized, sessionGrouping,
      sessionSidebarCollapsed: settings.sessionSidebarCollapsed === true,
      restoreOnStartup: settings.restoreOnStartup === true,
      terminalShortcuts: DEFAULT_TERMINAL_SHORTCUTS };
  } catch {
    return DEFAULT_UI_SETTINGS;
  }
}

function isLanguagePreference(value: unknown): value is LanguagePreference {
  return value === "system" || value === "en" || value === "zh-CN";
}

function isColorModePreference(value: unknown): value is ColorModePreference {
  return value === "system" || value === "light" || value === "dark";
}

function isSessionGrouping(value: unknown): value is SessionGrouping {
  return value === "directory" || value === "cli" || value === "none";
}

function isLaunchProfile(value: unknown): value is LaunchProfile {
  if (!value || typeof value !== "object") {
    return false;
  }

  const profile = value as Record<string, unknown>;
  return (
    typeof profile.id === "string" &&
    typeof profile.label === "string" &&
    isLaunchPreset(profile.preset) &&
    typeof profile.sessionName === "string" &&
    typeof profile.executable === "string" &&
    typeof profile.argumentsText === "string" &&
    typeof profile.workingDirectory === "string"
  );
}

function isLaunchPreset(value: unknown): value is LaunchPreset {
  return (
    value === "powerShell" ||
    value === "windowsPowerShell" ||
    value === "cmd" ||
    value === "codex" ||
    value === "claude" ||
    value === "custom"
  );
}

function clearLegacyPreferences() {
  try {
    localStorage.removeItem(LAUNCH_PROFILES_STORAGE_KEY);
    localStorage.removeItem(RECENT_DIRECTORIES_STORAGE_KEY);
    localStorage.removeItem(UI_SETTINGS_STORAGE_KEY);
  } catch {
    // The Rust preferences file is authoritative after migration.
  }
}

function createProfileId(): string {
  return globalThis.crypto?.randomUUID?.() ?? `profile-${Date.now()}`;
}

function createWorkspaceId(): string {
  return globalThis.crypto?.randomUUID?.() ?? `workspace-${Date.now()}`;
}

function directoryComparisonKey(directory: string): string {
  return directory.split("/").join("\\").toLowerCase();
}

export default App;
