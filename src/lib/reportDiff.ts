// What changed between two reports of the same machine: lines present only
// in the older one (gone) and only in the newer one (new). Order-insensitive
// on purpose — a report's sections are the same, their values are what move.
export function diffReportLines(before: string, after: string): { removed: string[]; added: string[] } {
  const clean = (t: string) => t.split("\n").map((l) => l.trimEnd()).filter((l) => l.trim() !== "");
  const a = clean(before);
  const b = clean(after);
  const inA = new Set(a);
  const inB = new Set(b);
  return { removed: a.filter((l) => !inB.has(l)), added: b.filter((l) => !inA.has(l)) };
}
