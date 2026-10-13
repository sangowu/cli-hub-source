import fs from "node:fs";
import assert from "node:assert/strict";
import ts from "typescript";

const source = fs.readFileSync(new URL("../src/terminalPreview.ts", import.meta.url), "utf8");
const js = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.ESNext } }).outputText;
const { readTerminalPreview, readTerminalPreviewCandidate, retainTerminalPreview } = await import("data:text/javascript;base64," + Buffer.from(js).toString("base64"));
function terminal(lines) {
  return { buffer: { active: { length: lines.length,
    getLine: i => ({ translateToString: () => typeof lines[i] === "string" ? lines[i] : lines[i].text,
      isWrapped: typeof lines[i] !== "string" }) } } };
}
function preview(lines) { return readTerminalPreview(terminal(lines)); }
assert.equal(preview(["● Earlier answer", "❯ Next question", "● Latest reply", "  It passed.",
  "─────────────────", "❯ Draft request", "⏵⏵ bypass permissions on · shift+tab to cycle"]), "Latest reply\nIt passed.");
assert.equal(preview(["• 修复完成。", "测试已通过。", "› 下一条输入", "gpt model · 92% context left"]), "修复完成。\n测试已通过。");
assert.equal(preview(["● Previous answer", "❯ Next task", "● Bash(npm run build)", "  tool output",
  "✻ Thinking…", "❯"]), "Previous answer");
assert.equal(preview(["Will install Pi.", "Installation complete.", "PS D:\\test> "]), "Will install Pi.\nInstallation complete.");
assert.equal(preview(["● A wrapped", { text: " response" }, "❯"]), "A wrapped response");
assert.equal(preview(["● " + "x".repeat(500), "❯"]).length, 360);
assert.equal(preview(["old", ...Array(240).fill("")]), "Waiting for terminal output…");
assert.equal(preview(["────────────────", "❯", "? for shortcuts"]), "Waiting for terminal output…");
let retained;
function frame(lines) {
  retained = retainTerminalPreview(retained, readTerminalPreviewCandidate(terminal(lines)));
  return retained.text;
}
assert.equal(frame(["● Previous answer", "❯"]), "Previous answer");
for (const lines of [[], ["⠋ Thinking…"], ["● Working…"], ["50%"], ["temporary tool output"], ["❯ draft"]]) {
  assert.equal(frame(lines), "Previous answer", `Redraw replaced reply: ${lines}`);
}
assert.equal(frame(["• New reply"]), "New reply");
assert.equal(frame(["• New reply grows", "with another line"]), "New reply grows\nwith another line");
assert.equal(preview(["● Working directory is ready"]), "Working directory is ready");
assert.equal(preview(["● Thinking about the design"]), "Thinking about the design");
retained = undefined;
assert.equal(frame(["shell output one"]), "shell output one");
assert.equal(frame(["⠹ Processing…", "75%"]), "shell output one");
assert.equal(frame(["shell output two"]), "shell output two");
assert.equal(frame([]), "shell output two");
console.log("PASS: replies survive blank/status/tool redraws, new replies stream, shell fallback updates, and scan/storage bounds hold");
