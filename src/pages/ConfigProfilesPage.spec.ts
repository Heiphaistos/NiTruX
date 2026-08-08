import { describe, it, expect, beforeEach, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import ConfigProfilesPage from "./ConfigProfilesPage.vue";
import { useProfilesStore } from "@/stores/profilesStore";
import { useThemeStore } from "@/stores/themeStore";
import { builtinThemes } from "@/themes/builtin";

describe("ConfigProfilesPage", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    document.documentElement.removeAttribute("style");
    localStorage.clear();
  });

  it("shows an empty state when no profile is saved yet", () => {
    const wrapper = mount(ConfigProfilesPage);
    expect(wrapper.text()).toContain("Aucun profil enregistré");
  });

  it("saving with a blank or whitespace-only name does nothing", async () => {
    const wrapper = mount(ConfigProfilesPage);
    await wrapper.find(".cp-save-row input").setValue("   ");
    const saveButton = wrapper.findAll("button").find((b) => b.text().includes("Enregistrer"))!;
    expect(saveButton.attributes("disabled")).toBeDefined();
    const profiles = useProfilesStore();
    expect(profiles.profiles).toEqual([]);
  });

  it("saves the current configuration under the typed name and lists it", async () => {
    const wrapper = mount(ConfigProfilesPage);
    await wrapper.find(".cp-save-row input").setValue("Bureau");
    const saveButton = wrapper.findAll("button").find((b) => b.text().includes("Enregistrer"))!;
    await saveButton.trigger("click");

    expect(wrapper.text()).toContain("Bureau");
    const profiles = useProfilesStore();
    expect(profiles.profiles.map((p) => p.name)).toEqual(["Bureau"]);
  });

  it("applying a listed profile calls profilesStore.apply and announces it", async () => {
    const themeStore = useThemeStore();
    themeStore.setTheme(builtinThemes[1]);
    const profiles = useProfilesStore();
    profiles.saveCurrentAs("Bureau");
    themeStore.setTheme(builtinThemes[0]); // switch away before applying

    const wrapper = mount(ConfigProfilesPage);
    const applyButton = wrapper.findAll("button").find((b) => b.text() === "Appliquer")!;
    await applyButton.trigger("click");

    expect(themeStore.active.id).toBe(builtinThemes[1].id);
    expect(wrapper.text()).toContain("Profil « Bureau » appliqué");
  });

  it("removing a profile takes it out of the list", async () => {
    const profiles = useProfilesStore();
    profiles.saveCurrentAs("Bureau");

    const wrapper = mount(ConfigProfilesPage);
    const removeButton = wrapper.findAll("button").find((b) => b.text() === "Supprimer")!;
    await removeButton.trigger("click");

    expect(wrapper.text()).not.toContain("Bureau");
    expect(profiles.profiles).toEqual([]);
  });

  it("shows an inline error (not a blocking native alert) when an imported file is invalid", async () => {
    const wrapper = mount(ConfigProfilesPage);
    const input = wrapper.find('input[type="file"]');
    const badFile = new File(["not valid json"], "bad-profile.json", { type: "application/json" });
    Object.defineProperty(input.element, "files", { value: [badFile], configurable: true });

    await input.trigger("change");
    await vi.waitFor(() => expect(wrapper.text()).toMatch(/JSON invalide/i));
  });

  it("importing a valid exported profile adds it to the list", async () => {
    const profiles = useProfilesStore();
    profiles.saveCurrentAs("Bureau");
    const json = profiles.exportProfile(profiles.profiles[0]);

    const wrapper = mount(ConfigProfilesPage);
    const input = wrapper.find('input[type="file"]');
    const file = new File([json], "Bureau.profile.json", { type: "application/json" });
    Object.defineProperty(input.element, "files", { value: [file], configurable: true });

    await input.trigger("change");
    await vi.waitFor(() => expect(profiles.profiles.length).toBe(2));
  });
});
