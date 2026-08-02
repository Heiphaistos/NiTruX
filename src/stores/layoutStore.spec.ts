import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useLayoutStore } from "./layoutStore";

describe("layoutStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("defaults to master-detail", () => {
    const store = useLayoutStore();
    expect(store.current).toBe("master-detail");
  });

  it("persists the chosen layout to localStorage", () => {
    const store = useLayoutStore();
    store.setLayout("bento");
    expect(localStorage.getItem("nitrux-layout")).toBe("bento");
  });

  it("falls back to master-detail when localStorage holds an invalid layout id", () => {
    localStorage.setItem("nitrux-layout", "not-a-real-layout");
    const store = useLayoutStore();
    expect(store.current).toBe("master-detail");
  });
});
