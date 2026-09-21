import { describe, it, expect } from "vitest";
import { parseHostsFile, toggleEntry, removeEntry, addEntry, isValidIp, isValidHostname } from "./hostsFile";

// A real Debian /etc/hosts, including the prose comment that must not be
// mistaken for an entry and a disabled entry that must be recognised.
const REAL_FILE = [
  "# Static table lookup for hostnames.",
  "# See hosts(5) for details.",
  "127.0.0.1\tlocalhost",
  "127.0.1.1\tdev-laptop dev-laptop.local",
  "# 192.168.1.50\tnas  # coupé le temps de la maintenance",
  "",
  "::1     ip6-localhost ip6-loopback",
].join("\n");

describe("hostsFile", () => {
  it("parses only the real entries, keeping prose comments out", () => {
    const entries = parseHostsFile(REAL_FILE);
    expect(entries.map((e) => e.ip)).toEqual(["127.0.0.1", "127.0.1.1", "192.168.1.50", "::1"]);
    expect(entries[1].hostnames).toEqual(["dev-laptop", "dev-laptop.local"]);
    expect(entries[2].disabled).toBe(true);
    expect(entries[2].comment).toBe("coupé le temps de la maintenance");
  });

  it("toggles one entry without touching any other line", () => {
    const entries = parseHostsFile(REAL_FILE);
    const updated = toggleEntry(REAL_FILE, entries[0]);
    expect(updated.split("\n")[2]).toBe("# 127.0.0.1\tlocalhost");
    // Everything else byte-identical.
    expect(updated.split("\n").filter((_, i) => i !== 2)).toEqual(REAL_FILE.split("\n").filter((_, i) => i !== 2));
  });

  it("re-enables a disabled entry", () => {
    const entries = parseHostsFile(REAL_FILE);
    const updated = toggleEntry(REAL_FILE, entries[2]);
    expect(updated).toContain("192.168.1.50\tnas # coupé le temps de la maintenance");
    expect(updated).not.toContain("# 192.168.1.50");
  });

  it("removes one entry and leaves the rest of the file intact", () => {
    const entries = parseHostsFile(REAL_FILE);
    const updated = removeEntry(REAL_FILE, entries[1]);
    expect(updated).not.toContain("dev-laptop");
    expect(updated).toContain("127.0.0.1\tlocalhost");
    expect(updated).toContain("ip6-localhost");
  });

  it("appends a new entry on its own line", () => {
    const updated = addEntry("127.0.0.1\tlocalhost", "10.0.0.5", ["serveur", "serveur.lan"], "labo");
    expect(updated).toBe("127.0.0.1\tlocalhost\n10.0.0.5\tserveur serveur.lan # labo\n");
    expect(parseHostsFile(updated)).toHaveLength(2);
  });

  it("validates addresses and hostnames before they reach the privileged write", () => {
    expect(isValidIp("192.168.1.1")).toBe(true);
    expect(isValidIp("::1")).toBe(true);
    expect(isValidIp("999.1.1.1")).toBe(false);
    expect(isValidIp("nas")).toBe(false);
    expect(isValidHostname("serveur.lan")).toBe(true);
    expect(isValidHostname("mauvais nom")).toBe(false);
    expect(isValidHostname("-debut")).toBe(false);
  });
});
