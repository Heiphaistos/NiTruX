import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useLayoutStore } from "./layoutStore";

describe("layoutStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("defaults to sidebar-classic", () => {
    const store = useLayoutStore();
    expect(store.current).toBe("sidebar-classic");
  });

  it("persists the chosen layout to localStorage", () => {
    const store = useLayoutStore();
    store.setLayout("bento");
    expect(localStorage.getItem("nitrux-layout")).toBe("bento");
  });
});
