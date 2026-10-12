import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import AdminHistoryPage from "./AdminHistoryPage.vue";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

describe("AdminHistoryPage", () => {
  it("lists actions with their outcome and error detail", async () => {
    invoke.mockResolvedValue([
      { timestamp_ms: 1, action: "set-dns", ok: false, detail: "authentification annulée" },
      { timestamp_ms: 0, action: "install-package pacman htop", ok: true, detail: "" },
    ]);
    const wrapper = mount(AdminHistoryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("install-package pacman htop"));
    expect(wrapper.text()).toContain("échec");
    expect(wrapper.text()).toContain("authentification annulée");
  });

  it("says so when nothing was run yet", async () => {
    invoke.mockResolvedValue([]);
    const wrapper = mount(AdminHistoryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucune action administrateur"));
  });
});
