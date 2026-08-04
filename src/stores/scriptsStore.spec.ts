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

  it("rejects adding a script whose name is already taken, instead of silently creating a duplicate", () => {
    // Without this guard, two scripts could share a name -- removeScript
    // filters by name, so deleting either one would silently delete BOTH
    // (real data loss), and Vue's :key="s.name" in ScriptsPage.vue would
    // warn/misbehave on the duplicate keys.
    const store = useScriptsStore();
    const first = store.addScript("backup", "echo first");
    expect(first.ok).toBe(true);

    const second = store.addScript("backup", "echo second");
    expect(second.ok).toBe(false);
    expect(store.scripts).toHaveLength(1);
    expect(store.scripts[0].content).toBe("echo first");
  });
});
