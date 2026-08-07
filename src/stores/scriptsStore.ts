import { defineStore } from "pinia";

const STORAGE_KEY = "nitrux-scripts";

export interface SavedScript {
  name: string;
  content: string;
}

function isSavedScript(value: unknown): value is SavedScript {
  if (typeof value !== "object" || value === null) return false;
  const v = value as Record<string, unknown>;
  return typeof v.name === "string" && typeof v.content === "string";
}

// Mirrors themeStore.ts/preferencesStore.ts's own strict per-field
// validation of persisted JSON, filtering out anything malformed instead of
// trusting the array shape alone -- a corrupted or stale-schema entry (e.g.
// missing `name`) would otherwise flow straight into ScriptsPage.vue's
// `:key="s.name"` as `undefined`, breaking this store's own single-identity
// guarantee that addScript/removeScript rely on.
function readPersistedScripts(): SavedScript[] {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (!stored) return [];
  try {
    const parsed: unknown = JSON.parse(stored);
    return Array.isArray(parsed) ? parsed.filter(isSavedScript) : [];
  } catch {
    return [];
  }
}

function persist(scripts: SavedScript[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(scripts));
}

export const useScriptsStore = defineStore("scripts", {
  state: (): { scripts: SavedScript[] } => ({ scripts: readPersistedScripts() }),
  actions: {
    // A duplicate name must never be allowed to reach the store: name is
    // the only identity this store has (used as Vue's :key in
    // ScriptsPage.vue and as removeScript's sole selector) -- two scripts
    // sharing a name would make removeScript delete both at once instead
    // of just the one the user intended.
    addScript(name: string, content: string): { ok: true } | { ok: false; error: string } {
      if (this.scripts.some((s) => s.name === name)) {
        return { ok: false, error: `Un script nommé « ${name} » existe déjà.` };
      }
      this.scripts.push({ name, content });
      persist(this.scripts);
      return { ok: true };
    },
    removeScript(name: string) {
      this.scripts = this.scripts.filter((s) => s.name !== name);
      persist(this.scripts);
    },
  },
});
