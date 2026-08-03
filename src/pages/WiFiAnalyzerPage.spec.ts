// src/pages/WiFiAnalyzerPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import WiFiAnalyzerPage from "./WiFiAnalyzerPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    wifi_networks: [
      { ssid: "WeakOpen", security: "", signal_percent: 20, connected: false },
      { ssid: "HomeWifi", security: "WPA2", signal_percent: 85, connected: true },
    ],
    listening_ports: [],
    dns_servers: [],
    hosts_file: "",
  }),
}));

describe("WiFiAnalyzerPage", () => {
  it("lists networks sorted by signal strength, strongest first", async () => {
    const wrapper = mount(WiFiAnalyzerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("HomeWifi"));
    const names = wrapper.findAll(".wifi-ssid").map((n) => n.text());
    expect(names).toEqual(["HomeWifi", "WeakOpen"]);
  });

  it("shows a danger badge for an open network and a success badge for WPA2", async () => {
    const wrapper = mount(WiFiAnalyzerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("HomeWifi"));
    expect(wrapper.find(".nx-badge--danger").exists()).toBe(true);
    expect(wrapper.find(".nx-badge--success").exists()).toBe(true);
  });

  it("shows an empty-state message instead of a blank page when no networks are visible", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValueOnce({ wifi_networks: [], listening_ports: [], dns_servers: [], hosts_file: "" });
    const wrapper = mount(WiFiAnalyzerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucun réseau Wi-Fi détecté."));
  });
});
