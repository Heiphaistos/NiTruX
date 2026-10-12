import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import GrubSettingsCard from "./GrubSettingsCard.vue";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));
const grub = (default_entry: string) => ({ default_entry, timeout_seconds: "5", distributor: "Debian", cmdline_default: "quiet" });

describe("GrubSettingsCard", () => {
  it("sets the menu timeout, and refuses more than 60 s in the page itself", async () => {
    invoke.mockResolvedValue("ok");
    const wrapper = mount(GrubSettingsCard, { props: { grub: grub("0") } });
    const apply = () => wrapper.findAll("button").find((b) => b.text() === "Appliquer")!;
    await wrapper.find("#gs-timeout").setValue("90");
    expect(apply().attributes("disabled")).toBeDefined();
    await wrapper.find("#gs-timeout").setValue("2");
    await apply().trigger("click");
    expect(invoke).toHaveBeenCalledWith("set_grub_timeout", { seconds: 2 });
    await vi.waitFor(() => expect(wrapper.text()).toContain("Délai réglé à 2 s."));
  });

  it("offers a one-time boot entry only once GRUB remembers the last choice", async () => {
    invoke.mockImplementation((cmd: string) =>
      Promise.resolve(cmd === "get_grub_entries" ? [{ index: 0, title: "Debian" }, { index: 1, title: "Windows" }] : "ok"),
    );
    const wrapper = mount(GrubSettingsCard, { props: { grub: grub("0") } });
    await wrapper.findAll("button").find((b) => b.text().startsWith("Afficher les entrées"))!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Windows"));
    const once = () => wrapper.findAll("button").filter((b) => b.text() === "Prochain démarrage")[1]!;
    expect(once().attributes("disabled")).toBeDefined();
    await wrapper.setProps({ grub: grub("saved") });
    await once().trigger("click");
    expect(invoke).toHaveBeenCalledWith("grub_reboot_once", { index: 1 });
  });
});
