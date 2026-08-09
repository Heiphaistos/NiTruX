import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import ProcessesPage from "./ProcessesPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_processes") return Promise.resolve([{ pid: 1234, name: "nitrux", cpu_percent: 2.5, memory_bytes: 104857600 }]);
    if (cmd === "get_systemd_services") return Promise.resolve(["ssh.service"]);
    if (cmd === "get_autostart_entries") return Promise.resolve([{ name: "nm-applet.desktop" }]);
    if (cmd === "get_scheduled_tasks") return Promise.resolve(["fwupd-refresh.timer"]);
    return Promise.resolve(null);
  }),
}));

describe("ProcessesPage", () => {
  it("shows processes, services, autostart entries, and scheduled tasks", async () => {
    const wrapper = mount(ProcessesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("nitrux"));
    expect(wrapper.text()).toContain("ssh.service");
    expect(wrapper.text()).toContain("nm-applet.desktop");
    expect(wrapper.text()).toContain("fwupd-refresh.timer");
  });

  it("filters the process list by name", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "get_processes") return Promise.resolve([
        { pid: 1, name: "nitrux", cpu_percent: 1, memory_bytes: 1000 },
        { pid: 2, name: "bash", cpu_percent: 0, memory_bytes: 500 },
      ]);
      return Promise.resolve([]);
    });
    const wrapper = mount(ProcessesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("nitrux"));
    const filterInput = wrapper.find("input");
    await filterInput.setValue("bash");
    expect(wrapper.text()).toContain("bash");
    expect(wrapper.text()).not.toContain("nitrux");
  });

  it("does not flash 'no service found' while get_systemd_services is still pending", async () => {
    // services/autostart/scheduledTasks (and processes, via filteredProcesses)
    // defaulted to [] rather than null, making their empty-state checks
    // indistinguishable from "still loading".
    const { invoke } = await import("@tauri-apps/api/core");
    let resolveServices!: (value: string[]) => void;
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_processes") return Promise.resolve([]);
      if (cmd === "get_systemd_services") return new Promise((resolve) => (resolveServices = resolve));
      if (cmd === "get_autostart_entries") return Promise.resolve([]);
      if (cmd === "get_scheduled_tasks") return Promise.resolve([]);
      return Promise.resolve(null);
    });

    const wrapper = mount(ProcessesPage);
    await wrapper.vm.$nextTick();
    expect(wrapper.text()).not.toContain("Aucun service systemd trouvé.");

    resolveServices(["ssh.service"]);
    await vi.waitFor(() => expect(wrapper.text()).toContain("ssh.service"));
  });

  it("shows process memory in French units (Mo), not the English 'MB'", async () => {
    // A prior test in this file overrides the shared `invoke` mock via
    // mockImplementation without resetting it afterwards, so the module-level
    // mock's fixture can't be relied on here -- set it explicitly instead.
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "get_processes") return Promise.resolve([{ pid: 1234, name: "nitrux", cpu_percent: 2.5, memory_bytes: 104857600 }]);
      return Promise.resolve([]);
    });
    const wrapper = mount(ProcessesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("nitrux"));
    expect(wrapper.text()).toContain("100.0 Mo");
    expect(wrapper.text()).not.toContain("MB");
  });

  it("shows an error message for a failing section without blocking the other three", async () => {
    // Regression guard: none of the 4 invoke() calls here were ever
    // guarded by try/catch -- a single failing section (e.g. no systemd,
    // so get_systemd_services rejects) previously left every later
    // `.value` assignment un-run, since a rejection anywhere in the
    // sequential onMounted body aborted the rest, with no error shown.
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "get_processes") return Promise.resolve([{ pid: 1, name: "nitrux", cpu_percent: 1, memory_bytes: 1000 }]);
      if (cmd === "get_systemd_services") return Promise.reject("systemctl introuvable");
      if (cmd === "get_autostart_entries") return Promise.resolve([{ name: "nm-applet.desktop" }]);
      if (cmd === "get_scheduled_tasks") return Promise.resolve(["fwupd-refresh.timer"]);
      return Promise.resolve(null);
    });
    const wrapper = mount(ProcessesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("systemctl introuvable"));
    // The other three sections must still have loaded successfully.
    expect(wrapper.text()).toContain("nitrux");
    expect(wrapper.text()).toContain("nm-applet.desktop");
    expect(wrapper.text()).toContain("fwupd-refresh.timer");
  });
});
