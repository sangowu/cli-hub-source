import type { Terminal } from "@xterm/xterm";

// A bounded view of rendered output, never a persisted conversation transcript.
const MAX_SCAN_LINES = 240;
const MAX_PREVIEW_CHARS = 360;
const replyStart = /^\s*[●•]\s+(.+)/u;
const prompt = /^\s*(?:[❯›>]|PS\s+[^>]*>|[A-Za-z]:\\[^>]*>)(?:\s|$)/u;
const chrome = /^\s*(?:[─━═╌┄│╭╰╮╯┌└┐┘├┤┬┴┼]+\s*$|[⏵⏸]|\? for shortcuts|.*\b(?:context left|auto-compact|bypass permissions|shift\+tab|esc to interrupt|tokens? ·)\b|[✻✽✢✳✶✺]\s)/iu;
const toolStart = /^(?:Bash|Read|Write|Edit|Update|Search|Glob|Grep|Task|TodoWrite|WebFetch|WebSearch)\(/u;
const activity = /^\s*(?:(?:[⠁-⣿◐◓◑◒|/\\-]\s*)?(?:Thinking|Working|Processing|Loading|Generating|思考中|处理中|加载中)[.…]*(?:\s+\d+(?:\.\d+)?s)?|[⠁-⣿◐◓◑◒]|(?:[█▓░#=-]+\s*)?\d{1,3}%)\s*$/iu;

export interface TerminalPreview {
  text: string;
  kind: "reply" | "output";
}

export function retainTerminalPreview(previous: TerminalPreview | undefined, candidate: TerminalPreview): TerminalPreview {
  // A redraw can temporarily remove the reply from the rendered screen. Keep
  // it until another reply appears, rather than replacing it with status/output.
  if (!candidate.text || (previous?.kind === "reply" && candidate.kind !== "reply")) {
    return previous ?? candidate;
  }
  return candidate;
}

export function readTerminalPreviewCandidate(terminal: Pick<Terminal, "buffer">): TerminalPreview {
  const buffer = terminal.buffer.active;
  const lines: string[] = [];
  for (let index = Math.max(0, buffer.length - MAX_SCAN_LINES); index < buffer.length; index++) {
    const line = buffer.getLine(index);
    const text = line?.translateToString(true).trimEnd() ?? "";
    if (line?.isWrapped && lines.length) lines[lines.length - 1] += text;
    else lines.push(text);
  }

  // Claude/Codex mark assistant output with a bullet. Stop at the next input
  // prompt or footer so a new draft/status redraw cannot replace the reply.
  let reply: string[] | null = null;
  let collecting = false;
  const output: string[] = [];
  for (const line of lines) {
    const start = replyStart.exec(line);
    const isActivity = activity.test(start?.[1] ?? line);
    if (start && !toolStart.test(start[1]) && !isActivity) {
      reply = [start[1]];
      collecting = true;
      continue;
    }
    if (prompt.test(line) || chrome.test(line) || isActivity || (start && toolStart.test(start[1]))) {
      collecting = false;
      continue;
    }
    if (collecting) reply!.push(line.trim());
    if (line.trim()) output.push(line.trim());
  }
  // Shell/custom programs have no universal message protocol. Use recent
  // non-chrome output when no recognized assistant reply is available.
  const replyText = reply?.join("\n").trim();
  return {
    text: (replyText || output.slice(-3).join("\n")).slice(0, MAX_PREVIEW_CHARS),
    kind: replyText ? "reply" : "output",
  };
}

export function readTerminalPreview(terminal: Pick<Terminal, "buffer">): string {
  return readTerminalPreviewCandidate(terminal).text || "Waiting for terminal output…";
}
