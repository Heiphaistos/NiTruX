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

  it("shows an empty-state message instead of a blank page when there are no accounts", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementationOnce((cmd: string) => (cmd === "get_user_accounts" ? Promise.resolve([]) : Promise.resolve(null)));
    const wrapper = mount(UserAccountsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucun compte utilisateur réel trouvé."));
  });

  it("shows an error message instead of a silently blank page when the backend call fails", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementationOnce((cmd: string) =>
      cmd === "get_user_accounts" ? Promise.reject("/etc/passwd illisible") : Promise.resolve(null),
    );
    const wrapper = mount(UserAccountsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("/etc/passwd illisible"));
  });

  it("does not flash 'no accounts found' while get_user_accounts is still pending", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    let resolveAccounts!: (value: unknown[]) => void;
    vi.mocked(invoke).mockImplementationOnce(() => new Promise((resolve) => (resolveAccounts = resolve)));

    const wrapper = mount(UserAccountsPage);
    await wrapper.vm.$nextTick();
    expect(wrapper.text()).not.toContain("Aucun compte utilisateur réel trouvé.");

    resolveAccounts([{ username: "dev", uid: 1000, home: "/home/dev", shell: "/bin/bash" }]);
    await vi.waitFor(() => expect(wrapper.text()).toContain("dev"));
  });
});
