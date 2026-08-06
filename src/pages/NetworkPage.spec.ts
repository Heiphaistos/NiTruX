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
    if (cmd === "get_network_interfaces") {
      return Promise.resolve([]);
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

  it("shows a 'not installed' message when Docker's binary is absent", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_network_snapshot") {
        return Promise.resolve({ wifi_networks: [], listening_ports: [], dns_servers: [], hosts_file: "127.0.0.1 localhost\n" });
      }
      if (cmd === "get_docker_snapshot") {
        return Promise.resolve({ available: false, installed: false, error: null, containers: [], images: [] });
      }
      return Promise.resolve(null);
    });
    const wrapper = mount(NetworkPage);
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("get_docker_snapshot"));
    const dockerTab = wrapper.findAll("button").find((b) => b.text() === "Docker")!;
    await dockerTab.trigger("click");
    expect(wrapper.text()).toContain("Docker n'est pas installé sur ce système.");
  });

  it("shows the real daemon error when Docker is installed but unreachable, distinct from 'not installed'", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_network_snapshot") {
        return Promise.resolve({ wifi_networks: [], listening_ports: [], dns_servers: [], hosts_file: "127.0.0.1 localhost\n" });
      }
      if (cmd === "get_docker_snapshot") {
        return Promise.resolve({
          available: false,
          installed: true,
          error: "failed to connect to the docker API at unix:///var/run/docker.sock: dial unix /var/run/docker.sock: connect: no such file or directory",
          containers: [],
          images: [],
        });
      }
      return Promise.resolve(null);
    });
    const wrapper = mount(NetworkPage);
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("get_docker_snapshot"));
    const dockerTab = wrapper.findAll("button").find((b) => b.text() === "Docker")!;
    await dockerTab.trigger("click");
    expect(wrapper.text()).toContain("Docker est installé mais injoignable");
    expect(wrapper.text()).toContain("no such file or directory");
    expect(wrapper.text()).not.toContain("n'est pas installé");
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

  it("shows each network interface's name, MAC address, and throughput", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_network_snapshot") {
        return Promise.resolve({ wifi_networks: [], listening_ports: [], dns_servers: [], hosts_file: "127.0.0.1 localhost\n" });
      }
      if (cmd === "get_docker_snapshot") return Promise.resolve({ available: false, containers: [], images: [] });
      if (cmd === "get_network_interfaces") {
        return Promise.resolve([
          { name: "eth0", mac_address: "aa:bb:cc:dd:ee:ff", rx_bytes_per_sec: 2048, tx_bytes_per_sec: 1024 },
          { name: "wlan0", mac_address: null, rx_bytes_per_sec: 0, tx_bytes_per_sec: 0 },
        ]);
      }
      return Promise.resolve(null);
    });
    const wrapper = mount(NetworkPage);
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("get_network_interfaces"));
    await vi.waitFor(() => expect(wrapper.text()).toContain("eth0"));
    expect(wrapper.text()).toContain("aa:bb:cc:dd:ee:ff");
    expect(wrapper.text()).toContain("2.0 Ko/s");
    expect(wrapper.text()).toContain("1.0 Ko/s");
    expect(wrapper.text()).toContain("wlan0");
    expect(wrapper.text()).toContain("MAC inconnue");
  });

  it("renders every listening-port row even when the same port appears twice (tcp+udp, or IPv4+IPv6)", async () => {
    // network.rs's real source (`ss -tulnp`) lists one row per socket, not
    // per unique port -- any service dual-bound on tcp+udp (systemd-resolved
    // on 53) or on both IPv4 and IPv6 (the common default for most modern
    // services) produces two lines sharing the same port number. Keying the
    // v-for on `p.port` alone is not guaranteed unique against this shape.
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_network_snapshot") {
        return Promise.resolve({
          wifi_networks: [],
          listening_ports: [
            { port: 53, process: "systemd-resolved" },
            { port: 53, process: "systemd-resolved" },
          ],
          dns_servers: [],
          hosts_file: "127.0.0.1 localhost\n",
        });
      }
      if (cmd === "get_docker_snapshot") return Promise.resolve({ available: false, containers: [], images: [] });
      return Promise.resolve(null);
    });
    const wrapper = mount(NetworkPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("systemd-resolved"));
    const portRows = wrapper.findAll(".net-row").filter((r) => r.text().includes("systemd-resolved"));
    expect(portRows.length).toBe(2);
  });
});
