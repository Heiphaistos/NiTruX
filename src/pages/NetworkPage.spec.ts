// src/pages/NetworkPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import NetworkPage from "./NetworkPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_network_snapshot") {
      return Promise.resolve({ wifi_networks: [], listening_ports: [], dns_servers: [], hosts_file: "127.0.0.1 localhost\n" });
    }
    if (cmd === "get_docker_snapshot") {
      return Promise.resolve({ available: false, containers: [], images: [] });
    }
    return Promise.resolve(null);
  }),
}));

describe("NetworkPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("invokes get_network_snapshot and get_docker_snapshot on mount, renders inside NxCard", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(NetworkPage);
    await vi.waitFor(() => expect(wrapper.find(".nx-card").exists()).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_network_snapshot");
    expect(invoke).toHaveBeenCalledWith("get_docker_snapshot");
  });

  it("shows empty-state messages when Docker is available but has no containers or images", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    // Override just this test's docker response; keep get_network_snapshot's
    // real shape (not null) so this doesn't leak a broken overview tab into
    // whichever test runs next in this file.
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_network_snapshot") {
        return Promise.resolve({ wifi_networks: [], listening_ports: [], dns_servers: [], hosts_file: "127.0.0.1 localhost\n" });
      }
      if (cmd === "get_docker_snapshot") return Promise.resolve({ available: true, containers: [], images: [] });
      return Promise.resolve(null);
    });
    const wrapper = mount(NetworkPage);
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("get_docker_snapshot"));
    const dockerTab = wrapper.findAll("button").find((b) => b.text() === "Docker")!;
    await dockerTab.trigger("click");
    expect(wrapper.text()).toContain("Aucun conteneur.");
    expect(wrapper.text()).toContain("Aucune image.");
  });

  it("calls write_hosts_file with the edited content on save", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(NetworkPage);
    await vi.waitFor(() => expect(wrapper.find("textarea").exists()).toBe(true));
    await wrapper.find("textarea").setValue("127.0.0.1 localhost\n127.0.1.1 test\n");
    const buttons = wrapper.findAll("button");
    const saveButtons = buttons.filter((b) => b.text() === "Enregistrer");
    await saveButtons[0].trigger("click");
    expect(invoke).toHaveBeenCalledWith("write_hosts_file", { content: "127.0.0.1 localhost\n127.0.1.1 test\n" });
  });

  it("pre-fills the DNS editor with content that already passes validate_dns_content's 'nameserver' requirement", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_network_snapshot") {
        return Promise.resolve({
          wifi_networks: [],
          listening_ports: [],
          dns_servers: ["1.1.1.1", "8.8.8.8"],
          hosts_file: "127.0.0.1 localhost\n",
        });
      }
      if (cmd === "get_docker_snapshot") return Promise.resolve({ available: false, containers: [], images: [] });
      return Promise.resolve(null);
    });
    const wrapper = mount(NetworkPage);
    await vi.waitFor(() => expect(wrapper.findAll("textarea").length).toBe(2));
    const dnsTextarea = wrapper.findAll("textarea")[1];
    expect((dnsTextarea.element as HTMLTextAreaElement).value).toBe("nameserver 1.1.1.1\nnameserver 8.8.8.8");
  });
});
