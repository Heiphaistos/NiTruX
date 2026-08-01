import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useScriptsStore } from "./scriptsStore";

describe("scriptsStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("starts empty and persists an added script across store instances", () => {
    const store = useScriptsStore();
    expect(store.scripts).toEqual([]);
    store.addScript("Lister /tmp", "ls -la /tmp");

    setActivePinia(createPinia());
    const reloaded = useScriptsStore();
    expect(reloaded.scripts).toEqual([{ name: "Lister /tmp", content: "ls -la /tmp" }]);
  });

  it("removes a script by name", () => {
    const store = useScriptsStore();
    store.addScript("A", "echo a");
    store.addScript("B", "echo b");
    store.removeScript("A");
    expect(store.scripts.map((s) => s.name)).toEqual(["B"]);
  });
});
