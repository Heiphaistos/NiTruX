import { describe, it, expect } from "vitest";
import { systemToolsCatalog } from "./systemToolsCatalog";

describe("systemToolsCatalog", () => {
  it("has a unique id for every entry", () => {
    const ids = systemToolsCatalog.map((t) => t.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("sets exactly one of command/privilegedAction per entry, never both or neither", () => {
    for (const tool of systemToolsCatalog) {
      const hasCommand = tool.command !== undefined;
      const hasPrivileged = tool.privilegedAction !== undefined;
      expect(hasCommand).not.toBe(hasPrivileged);
    }
  });

  // run_script (scripts.rs) discards stdout and returns an error on ANY
  // non-zero exit code (see subprocess::run_with_timeout) -- correct for a
  // generic "run this script" primitive, but wrong for a "du" one-liner
  // whose own exit code is a poor signal here: `du` exits 1 whenever it
  // hits a permission-denied subdirectory (extremely common under system
  // paths like /var/log, /etc, /home/*) or a path that doesn't exist
  // (equally common for the locale-dependent dual-path entries, e.g.
  // "~/Téléchargements ~/Downloads" -- only one of the two exists on any
  // given system) -- in both cases `du` still prints a real, useful
  // partial result on stdout, which run_script would silently throw away
  // in favor of a generic error. Reproduced live (2026-08-06, real
  // Debian VM): `du -sh ~/Téléchargements ~/Downloads 2>/dev/null` prints
  // a valid size for the one that exists but exits 1 -- confirmed on a
  // French-locale system, the common case. `|| true` neutralizes this for
  // every standalone du command not already piped into `sort`/`head`
  // (those are structurally safe already: the pipeline's exit code is the
  // last command's, not du's).
  it("never lets a standalone `du` command's own exit code turn a real (if partial) result into a discarded error", () => {
    for (const tool of systemToolsCatalog) {
      if (!tool.command?.trimStart().startsWith("du ")) continue;
      if (tool.command.includes("|")) continue; // piped into sort/head -- du's exit code isn't the pipeline's
      expect(tool.command.trimEnd().endsWith("|| true"), `${tool.id}: "${tool.command}"`).toBe(true);
    }
  });
});
