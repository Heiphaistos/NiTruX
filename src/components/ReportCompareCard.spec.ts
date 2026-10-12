import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import ReportCompareCard from "./ReportCompareCard.vue";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

describe("ReportCompareCard", () => {
  it("compares the previous report with the latest by default", async () => {
    invoke.mockImplementation((_cmd: string, args: { filename: string }) =>
      Promise.resolve(args.filename === "rapport-2.txt" ? "RAM: 5 Go\nCPU: 4" : "RAM: 2 Go\nCPU: 4"),
    );
    const wrapper = mount(ReportCompareCard, { props: { filenames: ["rapport-2.txt", "rapport-1.txt", "rapport-0.pdf"] } });
    await wrapper.findAll("button").find((b) => b.text() === "Comparer")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("+ RAM: 5 Go"));
    expect(wrapper.text()).toContain("- RAM: 2 Go");
    expect(wrapper.text()).not.toContain("rapport-0.pdf");
  });

  it("stays hidden with fewer than two text reports", () => {
    const wrapper = mount(ReportCompareCard, { props: { filenames: ["rapport-1.txt", "rapport-0.pdf"] } });
    expect(wrapper.text()).toBe("");
  });
});
