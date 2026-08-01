import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import DependenciesPage from "./DependenciesPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([{ binary: "/usr/bin/git", missing_library: "libfoo.so.3" }]),
}));

describe("DependenciesPage", () => {
  it("lists missing dependencies found on mount", async () => {
    const wrapper = mount(DependenciesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("/usr/bin/git"));
    expect(wrapper.text()).toContain("libfoo.so.3");
  });

  it("shows a clean-system message when nothing is missing", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce([]);
    const wrapper = mount(DependenciesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucune dépendance manquante"));
  });
});
