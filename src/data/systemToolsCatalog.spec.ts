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

  // Same root cause as the `du` case above, different tool shape: a
  // grep/pgrep command whose "no match" (exit 1, empty stdout) is a
  // routine, expected answer for a real chunk of users -- not a failure.
  // Reproduced live (2026-08-06, real Debian VM, a Hyper-V Gen2 guest):
  // `lspci | grep -i vga`/`-i audio` both exit 1 with empty output on
  // this VM (its synthetic VMBus display/audio path isn't PCI-classified
  // as VGA/audio at all -- a real, live case, not a hypothetical one, and
  // consistent with this same VMBus quirk already documented elsewhere in
  // this project's history); `pgrep -a openvpn`/`env | grep -i proxy`
  // both exit 1 with empty output for the ordinary case of a user with no
  // VPN running / no proxy configured, which is most users. Unlike the
  // `du` list above, this set isn't mechanically identifiable from the
  // command string alone (most grep-based entries target universally
  // present kernel/proc fields, like /proc/meminfo's MemAvailable, where
  // a match failing would be a genuine anomaly worth surfacing as an
  // error, not something to paper over) -- curated by id instead.
  const NO_MATCH_IS_A_VALID_ANSWER_IDS = [
    "gpu-info",
    "audio-devices",
    "cat-resolv-conf-dup",
    "curl-check-proxy",
    "mount-readonly",
    "xdg-desktop-portal-check",
    "openvpn-status-check",
    "curl-check-http3",
  ];

  it("keeps `|| true` on every catalog entry where a grep/pgrep 'no match' is a routine, valid answer", () => {
    for (const id of NO_MATCH_IS_A_VALID_ANSWER_IDS) {
      const tool = systemToolsCatalog.find((t) => t.id === id);
      expect(tool, `entry "${id}" should still exist in the catalog`).toBeDefined();
      expect(tool!.command?.trimEnd().endsWith("|| true"), `${id}: "${tool!.command}"`).toBe(true);
    }
  });

  it("includes a DNS cache flush action, a genuine gap against NiTriTe's equivalent Turbo Mode/Tools quick action", () => {
    // Unlike the du/grep cases above, a failing `resolvectl flush-caches`
    // (e.g. systemd-resolved not running) is a real, meaningful failure to
    // surface -- no `|| true` warranted here.
    const tool = systemToolsCatalog.find((t) => t.id === "resolvectl-flush-caches");
    expect(tool).toBeDefined();
    expect(tool!.command).toBe("resolvectl flush-caches");
    expect(tool!.category).toBe("reseau");
  });
});
