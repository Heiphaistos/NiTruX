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

  it("falls back to an empty list instead of loading malformed persisted entries", () => {
    // themeStore.ts::readPersistedTheme/readPersistedCustomThemes and
    // preferencesStore.ts::isPreferences both strictly validate every
    // parsed field before accepting persisted JSON, falling back to a safe
    // default on anything malformed. readPersistedScripts only checked
    // `Array.isArray(parsed)`, then cast the contents straight to
    // SavedScript[] with no per-item field validation -- a corrupted or
    // stale-schema localStorage entry (e.g. missing `name`) would flow
    // straight into ScriptsPage.vue's `:key="s.name"` as `undefined`,
    // breaking the store's own single identity guarantee that
    // addScript/removeScript rely on.
    localStorage.setItem("nitrux-scripts", JSON.stringify([{ foo: "bar" }, { name: 42, content: "echo x" }]));
    const store = useScriptsStore();
    expect(store.scripts).toEqual([]);
  });

  it("keeps only the well-formed entries from a persisted list that mixes valid and malformed scripts", () => {
    localStorage.setItem(
      "nitrux-scripts",
      JSON.stringify([{ name: "valid", content: "echo ok" }, { name: "bad" }, { content: "no name here" }]),
    );
    const store = useScriptsStore();
    expect(store.scripts).toEqual([{ name: "valid", content: "echo ok" }]);
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
