import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import PowerProfilesCard from "./PowerProfilesCard.vue";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

describe("PowerProfilesCard", () => {
  it("shows the active profile and switches on click", async () => {
    let active = "balanced";
    invoke.mockImplementation((cmd: string, args?: { profile: string }) => {
      if (cmd === "get_power_profiles") return Promise.resolve({ active, available: ["performance", "balanced", "power-saver"] });
      if (cmd === "set_power_profile") active = args!.profile;
      return Promise.resolve(null);
    });
    const wrapper = mount(PowerProfilesCard);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Équilibré (actif)"));
    await wrapper.findAll("button").find((b) => b.text() === "Performance")!.trigger("click");
    expect(invoke).toHaveBeenCalledWith("set_power_profile", { profile: "performance" });
    await vi.waitFor(() => expect(wrapper.text()).toContain("Performance (actif)"));
  });

  it("shows the missing-daemon error instead of an empty card", async () => {
    invoke.mockRejectedValue("powerprofilesctl introuvable — installez le paquet « power-profiles-daemon »");
    const wrapper = mount(PowerProfilesCard);
    await vi.waitFor(() => expect(wrapper.text()).toContain("power-profiles-daemon"));
  });
});
