// A list of catalog apps saved to a file, to reinstall the same set on
// another machine (or after a reinstall).
import { appCatalog } from "@/data/appCatalog";

const FORMAT = "nitrux-apps";

export function serializeAppSelection(ids: string[]): string {
  return JSON.stringify({ format: FORMAT, version: 1, apps: ids }, null, 2);
}

/** Catalog ids from a file the user picked: anything unknown is dropped,
 *  the file is never trusted to name a package directly. */
export function parseAppSelection(text: string): { ok: true; ids: string[]; unknown: number } | { ok: false; error: string } {
  let data: unknown;
  try {
    data = JSON.parse(text);
  } catch {
    return { ok: false, error: "fichier illisible : ce n'est pas du JSON" };
  }
  const apps = (data as { format?: unknown; apps?: unknown })?.apps;
  if ((data as { format?: unknown })?.format !== FORMAT || !Array.isArray(apps)) {
    return { ok: false, error: "ce fichier n'est pas une liste d'applications NiTruX" };
  }
  const known = new Set(appCatalog.map((e) => e.id));
  const ids = [...new Set(apps.filter((a): a is string => typeof a === "string" && known.has(a)))];
  return { ok: true, ids, unknown: apps.length - ids.length };
}
