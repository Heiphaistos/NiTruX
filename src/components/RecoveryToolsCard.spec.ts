import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import RecoveryToolsCard from "./RecoveryToolsCard.vue";
import { pendingTerminalCommand } from "@/lib/terminalCommand";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

describe("RecoveryToolsCard", () => {
  it("runs photorec under sudo in the integrated terminal", async () => {
    invoke.mockResolvedValue([{ binary: "testdisk", package: "testdisk", installed: true }]);
    const wrapper = mount(RecoveryToolsCard);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Lancer PhotoRec"));
    await wrapper.findAll("button").find((b) => b.text() === "Lancer PhotoRec")!.trigger("click");
    expect(pendingTerminalCommand.value).toBe("sudo photorec");
    expect(wrapper.emitted("navigate")).toEqual([["terminal"]]);
  });

  it("names the package to install when testdisk is missing", async () => {
    invoke.mockResolvedValue([{ binary: "testdisk", package: "testdisk", installed: false }]);
    const wrapper = mount(RecoveryToolsCard);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Non installé : paquet « testdisk »"));
  });
});
