import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import ScriptsPage from "./ScriptsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue("hello\n"),
}));

describe("ScriptsPage", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("shows an empty-state message before any script is saved", () => {
    const wrapper = mount(ScriptsPage);
    expect(wrapper.text()).toContain("Aucun script enregistré pour le moment.");
  });

  it("saves a new script and lists it", async () => {
    const wrapper = mount(ScriptsPage);
    await wrapper.find("input[placeholder*='Nom']").setValue("Test");
    await wrapper.find("textarea").setValue("echo hello");
    const saveButton = wrapper.findAll("button").find((b) => b.text() === "Enregistrer")!;
    await saveButton.trigger("click");
    expect(wrapper.text()).toContain("Test");
  });

  it("runs a saved script and shows its output", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(ScriptsPage);
    await wrapper.find("input[placeholder*='Nom']").setValue("Test");
    await wrapper.find("textarea").setValue("echo hello");
    await wrapper.findAll("button").find((b) => b.text() === "Enregistrer")!.trigger("click");
    const runButton = wrapper.findAll("button").find((b) => b.text() === "Exécuter")!;
    await runButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("hello"));
    expect(invoke).toHaveBeenCalledWith("run_script", { content: "echo hello" });
  });

  it("rejects a whitespace-only script name instead of saving an unidentifiable entry", async () => {
    // Regression guard for the actual gap: saveScript only checked
    // `newName.value === ""`, which a whitespace-only name (e.g. "   ")
    // passes -- the script would then save under an effectively blank,
    // indistinguishable name in the list.
    const wrapper = mount(ScriptsPage);
    await wrapper.find("input[placeholder*='Nom']").setValue("   ");
    await wrapper.find("textarea").setValue("echo hello");
    const saveButton = wrapper.findAll("button").find((b) => b.text() === "Enregistrer")!;
    await saveButton.trigger("click");
    expect(wrapper.findAll(".scr-item")).toHaveLength(0);
  });

  it("keeps one script's 'Exécuter' button enabled while an unrelated script is still running", async () => {
    // Regression guard for the actual bug: `running` used to be a single
    // shared ref (not per-script), so starting ANY script disabled every
    // OTHER script's button too, even completely unrelated ones -- the
    // same shared-state bug SystemToolsPage.vue already fixed for this
    // exact primitive.
    const { invoke } = await import("@tauri-apps/api/core");
    let resolveA!: (value: string) => void;
    vi.mocked(invoke).mockImplementationOnce(() => new Promise((resolve) => (resolveA = resolve)));

    const wrapper = mount(ScriptsPage);
    await wrapper.find("input[placeholder*='Nom']").setValue("A");
    await wrapper.find("textarea").setValue("sleep 30");
    await wrapper.findAll("button").find((b) => b.text() === "Enregistrer")!.trigger("click");

    await wrapper.find("input[placeholder*='Nom']").setValue("B");
    await wrapper.find("textarea").setValue("echo fast");
    await wrapper.findAll("button").find((b) => b.text() === "Enregistrer")!.trigger("click");

    // Located by their stable containing row (script name), not by button
    // text -- the button's own text flips to "En cours..." once running,
    // which would silently drop it from a `text() === "Exécuter"` filter.
    const items = () => wrapper.findAll(".scr-item");
    const runButtonFor = (name: string) => items().find((i) => i.text().includes(name))!.find("button");

    await runButtonFor("A").trigger("click"); // starts A, leaves its promise pending

    await vi.waitFor(() => expect(runButtonFor("A").attributes("disabled")).toBeDefined());
    expect(runButtonFor("B").attributes("disabled")).toBeUndefined();

    resolveA("done");
    await vi.waitFor(() => expect(runButtonFor("A").attributes("disabled")).toBeUndefined());
  });

  it("shows an error and keeps only the first script when saving a duplicate name", async () => {
    const wrapper = mount(ScriptsPage);
    await wrapper.find("input[placeholder*='Nom']").setValue("backup");
    await wrapper.find("textarea").setValue("echo first");
    await wrapper.findAll("button").find((b) => b.text() === "Enregistrer")!.trigger("click");

    await wrapper.find("input[placeholder*='Nom']").setValue("backup");
    await wrapper.find("textarea").setValue("echo second");
    await wrapper.findAll("button").find((b) => b.text() === "Enregistrer")!.trigger("click");

    expect(wrapper.text()).toMatch(/existe déjà/);
    expect(wrapper.findAll(".scr-item")).toHaveLength(1);
    expect(wrapper.text()).toContain("echo first");
    expect(wrapper.text()).not.toContain("echo second");
  });
});
