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

  it("renders every access point as a distinct row when several share the same SSID", async () => {
    // `nmcli dev wifi list` (the real backend source, network.rs) lists one
    // row per access point, not per unique network name -- a mesh system
    // (multiple APs broadcasting the same SSID for roaming) or several
    // hidden networks (which nmcli reports with an empty "" SSID) routinely
    // produce duplicate `ssid` values. Keying the v-for on `net.ssid` alone
    // is not guaranteed unique against this real data shape, which Vue's
    // key contract requires -- fixed defensively (index appended to the
    // key) even though the page's current single initial-mount lifecycle
    // (fetch once, no refresh) doesn't force Vue's reconciliation path that
    // would otherwise surface a visible "Duplicate keys" dev warning.
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValueOnce({
      wifi_networks: [
        { ssid: "MeshHome", security: "WPA2", signal_percent: 90, connected: true },
        { ssid: "MeshHome", security: "WPA2", signal_percent: 40, connected: false },
        { ssid: "", security: "", signal_percent: 60, connected: false },
        { ssid: "", security: "WPA2", signal_percent: 30, connected: false },
      ],
      listening_ports: [],
      dns_servers: [],
      hosts_file: "",
    });
    const wrapper = mount(WiFiAnalyzerPage);
    await vi.waitFor(() => expect(wrapper.findAll(".wifi-ssid").length).toBe(4));

    const signals = wrapper.findAll(".wifi-signal-label").map((n) => n.text());
    expect(signals).toEqual(["90%", "60%", "40%", "30%"]);
  });
});
