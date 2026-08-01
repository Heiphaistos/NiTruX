import { defineStore } from "pinia";

const STORAGE_KEY = "nitrux-scripts";

export interface SavedScript {
  name: string;
  content: string;
}

function readPersistedScripts(): SavedScript[] {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (!stored) return [];
  try {
    const parsed: unknown = JSON.parse(stored);
    return Array.isArray(parsed) ? (parsed as SavedScript[]) : [];
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
    addScript(name: string, content: string) {
      this.scripts.push({ name, content });
      persist(this.scripts);
    },
    removeScript(name: string) {
      this.scripts = this.scripts.filter((s) => s.name !== name);
      persist(this.scripts);
    },
  },
});
