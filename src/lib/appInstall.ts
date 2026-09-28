// One-click installation from any source, with automatic fallback.
//
// A catalog entry names ONE package id, which only exists on some
// distributions (`firefox-esr` is Debian-only, many apps are Flathub-only
// on openSUSE or Alpine...). Instead of failing there, the install tries
// the entry's own source first, then searches the machine's real sources
// (native repositories, Flathub, Snap Store) for the same app by name and
// installs the first exact match.
import { invoke } from "@tauri-apps/api/core";

export type InstallSource = "native" | "flatpak" | "snap";

export interface InstallCandidate {
  source: InstallSource;
  id: string;
  /** Native manager id (apt, dnf, pacman, zypper, apk, xbps) for "native". */
  manager?: string | null;
}

export interface SearchResult {
  source: InstallSource;
  manager: string | null;
  id: string;
  name: string;
  summary: string;
  version: string;
}

export interface SearchOutcome {
  results: SearchResult[];
  errors: string[];
}

export interface InstallSources {
  native: string | null;
  flatpak: boolean;
  flathub: boolean;
  snap: boolean;
  snapd_running: boolean;
}

export interface InstallAttempt {
  candidate: InstallCandidate;
  ok: boolean;
  message: string;
}

export type InstallResult =
  | { ok: true; used: InstallCandidate; output: string; attempts: InstallAttempt[] }
  | { ok: false; attempts: InstallAttempt[] };

export function sourceLabel(c: { source: InstallSource; manager?: string | null }): string {
  if (c.source === "flatpak") return "Flatpak";
  if (c.source === "snap") return "Snap";
  return c.manager ?? "natif";
}

export async function installCandidate(c: InstallCandidate): Promise<string> {
  if (c.source === "flatpak") return invoke<string>("install_flatpak_package", { appId: c.id });
  if (c.source === "snap") return invoke<string>("install_snap_package", { package: c.id });
  if (!c.manager) throw new Error("aucun gestionnaire de paquets natif détecté");
  return invoke<string>("install_package", { manager: c.manager, package: c.id });
}

const key = (c: InstallCandidate) => `${c.source}:${c.manager ?? ""}:${c.id}`;

/** Results that are the same app as `name`/`entryId`, best sources first. */
export function exactMatches(results: SearchResult[], name: string, entryId?: string): InstallCandidate[] {
  const wanted = new Set([name.toLowerCase(), (entryId ?? "").toLowerCase()].filter(Boolean));
  const order: Record<InstallSource, number> = { native: 0, flatpak: 1, snap: 2 };
  return results
    .filter((r) => {
      const last = r.id.split(".").pop()!.toLowerCase();
      return wanted.has(r.name.toLowerCase()) || wanted.has(r.id.toLowerCase()) || wanted.has(last);
    })
    .sort((a, b) => order[a.source] - order[b.source])
    .map((r) => ({ source: r.source, id: r.id, manager: r.manager }));
}

export async function installWithFallback(
  candidates: InstallCandidate[],
  fallback?: { name: string; entryId?: string },
): Promise<InstallResult> {
  const attempts: InstallAttempt[] = [];
  const tried = new Set<string>();

  async function tryAll(list: InstallCandidate[]): Promise<InstallResult | null> {
    for (const candidate of list) {
      if (tried.has(key(candidate))) continue;
      tried.add(key(candidate));
      try {
        const output = await installCandidate(candidate);
        attempts.push({ candidate, ok: true, message: output });
        return { ok: true, used: candidate, output, attempts };
      } catch (e) {
        attempts.push({ candidate, ok: false, message: String(e) });
      }
    }
    return null;
  }

  const direct = await tryAll(candidates);
  if (direct) return direct;
  if (fallback) {
    try {
      const outcome = await invoke<SearchOutcome | null>("search_packages", { query: fallback.name });
      const found = await tryAll(exactMatches(outcome?.results ?? [], fallback.name, fallback.entryId));
      if (found) return found;
    } catch (e) {
      attempts.push({ candidate: { source: "native", id: fallback.name }, ok: false, message: `recherche : ${e}` });
    }
  }
  return { ok: false, attempts };
}

export function describeFailure(attempts: InstallAttempt[]): string {
  if (attempts.length === 0) return "aucune source d'installation disponible pour cette application";
  return attempts.map((a) => `${sourceLabel(a.candidate)} (${a.candidate.id}) : ${a.message}`).join("\n");
}
