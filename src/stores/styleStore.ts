import { defineStore } from "pinia";
import type { StyleId } from "@/types/style";
import { styleRegistry } from "@/styles/registry";

const STORAGE_KEY = "nitrux-style";
const DEFAULT_STYLE: StyleId = "glass-glow";

function isValidStyleId(value: string | null): value is StyleId {
  return value !== null && styleRegistry.some((s) => s.id === value);
}

function readPersistedStyle(): StyleId {
  const stored = localStorage.getItem(STORAGE_KEY);
  return isValidStyleId(stored) ? stored : DEFAULT_STYLE;
}

function applyToDom(id: StyleId) {
  document.documentElement.dataset.nxStyle = id;
}

export const useStyleStore = defineStore("style", {
  state: () => ({
    current: readPersistedStyle(),
  }),
  actions: {
    setStyle(id: StyleId) {
      this.current = id;
      localStorage.setItem(STORAGE_KEY, id);
      applyToDom(id);
    },
  },
});
