// src/pages/DnsSwitcherPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import DnsSwitcherPage from "./DnsSwitcherPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_network_snapshot") {
      return Promise.resolve({ wifi_networks: [], listening_ports: [], dns_servers: ["1.1.1.1"], hosts_file: "" });
    }
    if (cmd === "set_dns_servers") return Promise.resolve(null);
    return Promise.resolve(null);
  }),
}));

describe("DnsSwitcherPage", () => {
  it("applies the Cloudflare preset via set_dns_servers with valid nameserver-prefixed lines", async () => {
    // Bare IPs (no "nameserver " prefix) fail the real backend's
    // validate_dns_content, which requires at least one "nameserver " line
    // -- confirmed against network_write.rs. This page's preset buttons and
    // manual textarea both intentionally work with bare IPs (that's the
    // whole point of a "one server per line" UX), so the "nameserver "
    // prefix must be added before sending, not left for the user to type.
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(DnsSwitcherPage);
    const button = wrapper.findAll("button").find((b) => b.text().includes("Cloudflare"))!;
    await button.trigger("click");
    await vi.waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("set_dns_servers", { content: "nameserver 1.1.1.1\nnameserver 1.0.0.1" }),
    );
  });

  it("applies a manually entered DNS list via set_dns_servers with valid nameserver-prefixed lines", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(DnsSwitcherPage);
    await wrapper.find("textarea").setValue("9.9.9.9");
    const button = wrapper.findAll("button").find((b) => b.text() === "Appliquer")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("set_dns_servers", { content: "nameserver 9.9.9.9" }));
  });

  it("skips blank lines from the manual textarea instead of sending an empty nameserver entry", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(DnsSwitcherPage);
    await wrapper.find("textarea").setValue("9.9.9.9\n\n149.112.112.112\n");
    const button = wrapper.findAll("button").find((b) => b.text() === "Appliquer")!;
    await button.trigger("click");
    await vi.waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("set_dns_servers", {
        content: "nameserver 9.9.9.9\nnameserver 149.112.112.112",
      }),
    );
  });
});
