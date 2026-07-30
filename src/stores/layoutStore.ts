import { defineStore } from "pinia";
import type { LayoutId } from "@/types/layout";
import { layoutRegistry } from "@/layouts/registry";

const STORAGE_KEY = "nitrux-layout";
const DEFAULT_LAYOUT: LayoutId = "sidebar-classic";

function isValidLayoutId(value: string | null): value is LayoutId {
  return value !== null && layoutRegistry.some((l) => l.id === value);
}

function readPersistedLayout(): LayoutId {
  const stored = localStorage.getItem(STORAGE_KEY);
  return isValidLayoutId(stored) ? stored : DEFAULT_LAYOUT;
}

export const useLayoutStore = defineStore("layout", {
  state: () => ({
    current: readPersistedLayout(),
  }),
  actions: {
    setLayout(id: LayoutId) {
      this.current = id;
      localStorage.setItem(STORAGE_KEY, id);
    },
  },
});
