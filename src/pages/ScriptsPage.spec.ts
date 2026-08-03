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
});
