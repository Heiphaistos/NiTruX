import { defineStore } from "pinia";
import type { LayoutId } from "@/types/layout";

const STORAGE_KEY = "nitrux-layout";

export const useLayoutStore = defineStore("layout", {
  state: () => ({
    current: (localStorage.getItem(STORAGE_KEY) as LayoutId | null) ?? ("sidebar-classic" as LayoutId),
  }),
  actions: {
    setLayout(id: LayoutId) {
      this.current = id;
      localStorage.setItem(STORAGE_KEY, id);
    },
  },
});
