import fs from "node:fs";
import assert from "node:assert/strict";
import ts from "typescript";

const js = ts.transpileModule(fs.readFileSync(new URL("../src/sessionHistory.ts", import.meta.url), "utf8"),
  { compilerOptions: { module: ts.ModuleKind.ESNext } }).outputText;
const { sessionHistoryJson, readSessionHistory, reconcileSessionHistory } =
  await import("data:text/javascript;base64," + Buffer.from(js).toString("base64"));
const record = (historyKey, overrides = {}) => ({ historyKey, id: 1, processId: 42,
  pageId: "workspace-default", preset: "cmd", name: "Test", workingDirectory: "D:\\test",
  workingDirectoryKey: "d:\\test", status: "running", exitCode: null, ...overrides });

const live = record("new-key");
const ended = record("ended", { id: 2, status: "exited", exitCode: 0 });
let reconciled = reconcileSessionHistory([record("original"), ended], [live]);
assert.equal(reconciled.active[0].historyKey, "original");
assert.equal(reconciled.ended[0].historyKey, "ended");
reconciled = reconcileSessionHistory([record("original")], [record("new-key", { id: 9, processId: 100 })]);
assert.equal(reconciled.active[0].historyKey, "original", "Startup restoration must not duplicate a card");
assert.equal(reconciled.ended.length, 0);
reconciled = reconcileSessionHistory([record("old")], []);
assert.equal(reconciled.ended[0].status, "exited", "Missing processes must not be shown as live");
reconciled = reconcileSessionHistory([ended], [record("new-key", { id: 2 })]);
assert.equal(reconciled.ended.length, 1, "A reused process id must not replace a stopped record");

const serialized = sessionHistoryJson([{ ...ended, id: -1, displayId: 2,
  preview: "SECRET_OUTPUT", output: [1, 2], error: "SECRET_ERROR",
  arguments: ["SECRET_ARGUMENT"], environment: { SECRET_ENV: "hidden" } }]);
assert(!serialized.includes("SECRET"));
const stored = readSessionHistory({ getItem: () => serialized });
assert.equal(stored[0].id, 2);
assert.equal(stored[0].exitCode, 0);
assert.deepEqual(readSessionHistory({ getItem: () => null }), []);
assert.throws(() => readSessionHistory({ getItem: () => '{"version":99,"records":[]}' }));
assert.throws(() => readSessionHistory({ getItem: () => sessionHistoryJson([ended, ended]) }));
console.log("PASS: live/ended/relaunched records, PID reuse, metadata-only storage, original ids, invalid history protection");
