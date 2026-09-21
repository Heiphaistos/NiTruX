// Line-level view of /etc/hosts, so the UI can toggle or delete one entry
// instead of only handing the whole file back as a blob of text.
//
// Everything here is a pure string transformation: the file is still read
// through `get_network_snapshot` and written through `write_hosts_file`
// (the existing privileged path). No new backend surface, and unknown
// lines -- comments, blank lines, anything unparseable -- are preserved
// verbatim, so round-tripping a file NiTruX did not write never destroys
// content it did not understand.

export interface HostsEntry {
  /** Index of the line in the original file, the identity used to edit it. */
  line: number;
  ip: string;
  hostnames: string[];
  /** A commented-out entry: still in the file, not resolved. */
  disabled: boolean;
  comment: string | null;
}

const ENTRY_PATTERN = /^(\s*)(#\s*)?([0-9a-fA-F:.]+)\s+([^#]+?)\s*(#.*)?$/;

function parseLine(raw: string, line: number): HostsEntry | null {
  const match = raw.match(ENTRY_PATTERN);
  if (!match) return null;
  const [, , commentMarker, ip, hostPart, trailing] = match;
  // An IP is the only thing that makes a line an entry; a plain prose
  // comment ("# Static table lookup for hostnames") must not become one.
  if (!/\d/.test(ip) && !ip.includes(":")) return null;
  const hostnames = hostPart.trim().split(/\s+/).filter(Boolean);
  if (hostnames.length === 0) return null;
  return {
    line,
    ip,
    hostnames,
    disabled: Boolean(commentMarker),
    comment: trailing ? trailing.replace(/^#\s*/, "") : null,
  };
}

export function parseHostsFile(content: string): HostsEntry[] {
  return content
    .split("\n")
    .map(parseLine)
    .filter((entry): entry is HostsEntry => entry !== null);
}

export function formatEntry(entry: Omit<HostsEntry, "line">): string {
  const body = `${entry.ip}\t${entry.hostnames.join(" ")}${entry.comment ? ` # ${entry.comment}` : ""}`;
  return entry.disabled ? `# ${body}` : body;
}

/** Replaces one line, leaving every other byte of the file untouched. */
function replaceLine(content: string, line: number, replacement: string | null): string {
  const lines = content.split("\n");
  if (line < 0 || line >= lines.length) return content;
  if (replacement === null) {
    lines.splice(line, 1);
  } else {
    lines[line] = replacement;
  }
  return lines.join("\n");
}

export function toggleEntry(content: string, entry: HostsEntry): string {
  return replaceLine(content, entry.line, formatEntry({ ...entry, disabled: !entry.disabled }));
}

export function removeEntry(content: string, entry: HostsEntry): string {
  return replaceLine(content, entry.line, null);
}

export function addEntry(content: string, ip: string, hostnames: string[], comment: string | null = null): string {
  const line = formatEntry({ ip, hostnames, disabled: false, comment });
  return content.endsWith("\n") || content === "" ? `${content}${line}\n` : `${content}\n${line}\n`;
}

/** `ip` is validated here rather than trusted: it ends up in a privileged write. */
export function isValidIp(ip: string): boolean {
  const v4 = ip.match(/^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/);
  if (v4) return v4.slice(1).every((part) => Number(part) <= 255);
  return /^[0-9a-fA-F:]+$/.test(ip) && ip.includes(":");
}

export function isValidHostname(name: string): boolean {
  return (
    name.length <= 253 &&
    /^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?)*$/.test(name)
  );
}
