import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useStyleStore } from "./styleStore";
import { styleRegistry } from "@/styles/registry";

describe("styleStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
    document.documentElement.removeAttribute("data-nx-style");
  });

  it("defaults to glass-glow when nothing persisted", () => {
    const store = useStyleStore();
    expect(store.current).toBe("glass-glow");
  });

  it("setStyle updates state, persists to localStorage, and sets the data attribute", () => {
    const store = useStyleStore();
    store.setStyle("brutalism");
    expect(store.current).toBe("brutalism");
    expect(localStorage.getItem("nitrux-style")).toBe("brutalism");
    expect(document.documentElement.dataset.nxStyle).toBe("brutalism");
  });

  it("falls back to the default when localStorage holds an unknown style id", () => {
    localStorage.setItem("nitrux-style", "not-a-real-style");
    const store = useStyleStore();
    expect(store.current).toBe("glass-glow");
  });

  it.each(styleRegistry.map((s) => s.id))("accepts %s as a valid style id", (id) => {
    const store = useStyleStore();
    store.setStyle(id);
    expect(store.current).toBe(id);
  });
});
