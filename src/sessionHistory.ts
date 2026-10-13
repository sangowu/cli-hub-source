export const SESSION_HISTORY_STORAGE_KEY = "cli-hub.session-records.v1";

export interface SavedSessionRecord {
  historyKey: string;
  id: number;
  processId: number | null;
  pageId: string;
  preset: "powerShell" | "windowsPowerShell" | "cmd" | "codex" | "claude" | "custom";
  name: string;
  workingDirectory: string;
  workingDirectoryKey: string;
  status: "running" | "exited" | "failed";
  exitCode: number | null;
}

export function newSessionHistoryKey(): string {
  return globalThis.crypto?.randomUUID?.() ?? `session-${Date.now()}-${Math.random()}`;
}

// Deliberately exclude previews, terminal output, errors, arguments and environment.
export function sessionHistoryJson(records: (SavedSessionRecord & { displayId?: number })[]): string {
  return JSON.stringify({ version: 1, records: records.map((record) => ({
    historyKey: record.historyKey, id: record.displayId ?? record.id,
    processId: record.processId, pageId: record.pageId, preset: record.preset,
    name: record.name, workingDirectory: record.workingDirectory,
    workingDirectoryKey: record.workingDirectoryKey, status: record.status, exitCode: record.exitCode,
  })) });
}

export function readSessionHistory(storage: Pick<Storage, "getItem">): SavedSessionRecord[] {
  const raw = storage.getItem(SESSION_HISTORY_STORAGE_KEY);
  if (!raw) return [];
  const parsed = JSON.parse(raw);
  if (parsed.version !== 1 || !Array.isArray(parsed.records) ||
      !parsed.records.every((record: SavedSessionRecord) => record &&
        [record.historyKey, record.pageId, record.name, record.workingDirectory, record.workingDirectoryKey]
          .every((value) => typeof value === "string") &&
        Number.isSafeInteger(record.id) && record.id >= 0 &&
        (record.processId === null || Number.isSafeInteger(record.processId)) &&
        (record.exitCode === null || Number.isSafeInteger(record.exitCode)) &&
        ["running", "exited", "failed"].includes(record.status) &&
        ["powerShell", "windowsPowerShell", "cmd", "codex", "claude", "custom"].includes(record.preset)) ||
      new Set(parsed.records.map((record: SavedSessionRecord) => record.historyKey)).size !== parsed.records.length) {
    throw new Error("Invalid saved session records; existing data was left untouched");
  }
  return parsed.records;
}

export function reconcileSessionHistory(saved: SavedSessionRecord[], live: SavedSessionRecord[]) {
  const remaining = [...saved];
  const active = live.map((record) => {
    const sameLaunch = (candidate: SavedSessionRecord) => candidate.status === "running" &&
      candidate.pageId === record.pageId && candidate.preset === record.preset &&
      candidate.name === record.name && candidate.workingDirectoryKey === record.workingDirectoryKey;
    let index = remaining.findIndex((candidate) => sameLaunch(candidate) &&
      candidate.id === record.id && candidate.processId === record.processId);
    // Startup restoration may relaunch a saved running card with a new PID/id.
    if (index < 0) index = remaining.findIndex(sameLaunch);
    if (index < 0) return record;
    const [previous] = remaining.splice(index, 1);
    return { ...record, historyKey: previous.historyKey };
  });
  const ended = remaining.map((record) => record.status === "running"
    ? { ...record, status: "exited" as const, exitCode: null } : record);
  return { active, ended };
}
