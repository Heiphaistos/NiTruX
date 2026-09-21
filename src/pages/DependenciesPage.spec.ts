import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import DependenciesPage from "./DependenciesPage.vue";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const TOOLS = [
  { binary: "sensors", package: "lm-sensors", feature: "Températures", installed: false },
  { binary: "lspci", package: "pciutils", feature: "Composants PCI, Pilotes", installed: true },
];

async function mockInvoke(overrides: Record<string, unknown> = {}) {
  const { invoke } = await import("@tauri-apps/api/core");
  vi.mocked(invoke).mockImplementation((cmd: string) => {
    // Every command this page calls gets an answer of the right shape:
    // a mock that only handles the command under test makes the others
    // fall through to `undefined` and breaks unrelated assertions.
    const answers: Record<string, unknown> = {
      scan_missing_dependencies: [],
      detect_native_manager: "apt",
      check_required_tools: TOOLS,
      install_package: "installé",
      ...overrides,
    };
    const answer = answers[cmd];
    return answer instanceof Error ? Promise.reject(answer) : Promise.resolve(answer);
  });
  return invoke;
}

describe("DependenciesPage — outils système requis", () => {
  beforeEach(() => vi.clearAllMocks());

  it("lists only the missing tools, with the feature they disable and their package", async () => {
    await mockInvoke();
    const wrapper = mount(DependenciesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("sensors"));
    expect(wrapper.text()).toContain("Températures");
    expect(wrapper.text()).toContain("lm-sensors");
    // An installed tool is not a problem to report.
    expect(wrapper.text()).not.toContain("pciutils");
    expect(wrapper.text()).toContain("1 manquant(s) sur 2");
  });

  it("installs a missing tool through the detected package manager", async () => {
    const invoke = await mockInvoke();
    const wrapper = mount(DependenciesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("sensors"));
    const install = wrapper.findAll("button").find((b) => b.text() === "Installer")!;
    await install.trigger("click");
    await vi.waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("install_package", { manager: "apt", package: "lm-sensors" }),
    );
  });

  it("shows the install error instead of silently doing nothing", async () => {
    await mockInvoke({ install_package: new Error("pkexec: autorisation refusée") });
    const wrapper = mount(DependenciesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("sensors"));
    const install = wrapper.findAll("button").find((b) => b.text() === "Installer")!;
    await install.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("autorisation refusée"));
  });

  it("says so when nothing is missing", async () => {
    await mockInvoke({ check_required_tools: [{ ...TOOLS[0], installed: true }] });
    const wrapper = mount(DependenciesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Tous les outils utilisés par NiTruX sont installés"));
  });
});
