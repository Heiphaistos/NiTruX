import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import LogsPage from "./LogsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([
    { priority: 3, message: "disk failure imminent", unit: "smartd" },
  ]),
}));

describe("LogsPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("invokes get_recent_logs with limit 200 and renders the entry inside an NxCard", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(LogsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("disk failure imminent"));
    expect(invoke).toHaveBeenCalledWith("get_recent_logs", { limit: 200 });
    expect(wrapper.find(".nx-card").exists()).toBe(true);
  });
});
