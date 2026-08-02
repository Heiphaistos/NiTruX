import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import UserAccountsPage from "./UserAccountsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_user_accounts") {
      return Promise.resolve([{ username: "dev", uid: 1000, home: "/home/dev", shell: "/bin/bash" }]);
    }
    return Promise.resolve(null);
  }),
}));

describe("UserAccountsPage", () => {
  it("lists real user accounts", async () => {
    const wrapper = mount(UserAccountsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("dev"));
    expect(wrapper.text()).toContain("/home/dev");
    expect(wrapper.text()).toContain("/bin/bash");
  });
});
