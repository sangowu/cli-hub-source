import fs from "node:fs";
import assert from "node:assert/strict";
import ts from "typescript";

const source = fs.readFileSync(new URL("../src/terminalShortcuts.ts", import.meta.url), "utf8");
const js = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.ESNext } }).outputText;
const { DEFAULT_TERMINAL_SHORTCUTS: defaults, shortcutFromEvent, terminalNewlineData, conflictingShortcut } =
  await import("data:text/javascript;base64," + Buffer.from(js).toString("base64"));
const event = (extra = {}) => ({
  key: "Enter", keyCode: 13, shiftKey: true, ctrlKey: false, altKey: false,
  metaKey: false, isComposing: false, getModifierState: () => false, ...extra,
});

assert.equal(defaults.newline, "Shift+Enter");
assert.equal(shortcutFromEvent(event()), "Shift+Enter");
assert.equal(shortcutFromEvent(event({ shiftKey: false })), null);
assert.equal(terminalNewlineData(event(), "codex", defaults.newline), "\x1b\r");
assert.equal(terminalNewlineData(event(), "claude", defaults.newline), "\n");
for (const preset of ["cmd", "powerShell", "windowsPowerShell", "custom"])
  assert.equal(terminalNewlineData(event(), preset, defaults.newline), null);
for (const override of [{ isComposing: true }, { keyCode: 229 }, { getModifierState: () => true }])
  assert.equal(terminalNewlineData(event(override), "codex", defaults.newline), null);

const custom = { ...defaults, newline: "Ctrl+Enter" };
assert.equal(terminalNewlineData(event(), "codex", custom.newline), null);
for (const preset of ["codex", "claude"])
  assert.notEqual(terminalNewlineData(event({ shiftKey: false, ctrlKey: true }), preset, custom.newline), null);
assert.equal(conflictingShortcut(custom), false);
assert.equal(conflictingShortcut({ ...defaults, newline: defaults.interrupt }), true);
assert.equal(terminalNewlineData(event({ key: "N", ctrlKey: true, shiftKey: false }), "codex", "Ctrl+N"), "\x1b\r");
console.log("PASS: default/custom newline bindings, retired key, conflicts, native presets, and IME guards");
