export interface TerminalShortcuts {
  copy: string;
  paste: string;
  pasteAlternate: string;
  interrupt: string;
  newline: string;
  smartCopy: boolean;
}

export const DEFAULT_TERMINAL_SHORTCUTS: TerminalShortcuts = {
  copy: "Ctrl+Shift+C", paste: "Ctrl+V", pasteAlternate: "Ctrl+Shift+V",
  interrupt: "Ctrl+C", newline: "Shift+Enter", smartCopy: true,
};

export function shortcutFromEvent(event: KeyboardEvent): string | null {
  if (event.isComposing || event.keyCode === 229 || event.getModifierState("AltGraph") ||
      ["Control", "Shift", "Alt", "Meta"].includes(event.key)) return null;
  if (!event.ctrlKey && !event.altKey && !event.metaKey &&
      !(event.shiftKey && event.key === "Enter")) return null;
  const key = event.key.length === 1 ? event.key.toUpperCase() : event.key;
  if (!/^[A-Z0-9]$|^F\d{1,2}$|^(Enter|Insert|Delete|Home|End|PageUp|PageDown|ArrowUp|ArrowDown|ArrowLeft|ArrowRight)$/.test(key)) return null;
  return [event.ctrlKey && "Ctrl", event.altKey && "Alt", event.shiftKey && "Shift",
    event.metaKey && "Meta", key].filter(Boolean).join("+");
}

// Legacy Enter encoding drops Shift. Only adapt known agent presets: shells
// and custom programs can interpret a newline as command submission.
export function terminalNewlineData(event: KeyboardEvent, preset: string, shortcut: string): string | null {
  if (shortcutFromEvent(event) !== shortcut) return null;
  // Alt+Enter survives Windows ConPTY's legacy key translation; CSI-u does not.
  if (preset === "codex") return "\x1b\r";
  if (preset === "claude") return "\n";
  return null;
}

export function conflictingShortcut(shortcuts: TerminalShortcuts): boolean {
  const keys = [shortcuts.copy, shortcuts.paste, shortcuts.pasteAlternate, shortcuts.interrupt, shortcuts.newline];
  return new Set(keys).size !== keys.length;
}
