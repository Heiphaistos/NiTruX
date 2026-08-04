// src/pages/ReportGeneratorPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import ReportGeneratorPage from "./ReportGeneratorPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "generate_system_report" && args?.format === "json") {
      return Promise.resolve('{"generated_at":"epoch:123"}');
    }
    if (cmd === "generate_system_report" && args?.format === "markdown") {
      return Promise.resolve("# Rapport système NiTruX");
    }
    return Promise.resolve(null);
  }),
}));

describe("ReportGeneratorPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("generates a JSON report by default and shows a preview", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(ReportGeneratorPage);
    const button = wrapper.findAll("button").find((b) => b.text() === "Générer")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("generated_at"));
    expect(invoke).toHaveBeenCalledWith("generate_system_report", { format: "json" });
  });

  it("generates a Markdown report when that format is selected", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(ReportGeneratorPage);
    const select = wrapper.find("select");
    await select.setValue("markdown");
    const button = wrapper.findAll("button").find((b) => b.text() === "Générer")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Rapport système NiTruX"));
    expect(invoke).toHaveBeenCalledWith("generate_system_report", { format: "markdown" });
  });

  it("enables the download button only after a report has been generated", async () => {
    const wrapper = mount(ReportGeneratorPage);
    const downloadButtonBefore = wrapper.findAll("button").find((b) => b.text() === "Télécharger");
    expect(downloadButtonBefore?.attributes("disabled")).toBeDefined();
    const generateButton = wrapper.findAll("button").find((b) => b.text() === "Générer")!;
    await generateButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("generated_at"));
    const downloadButtonAfter = wrapper.findAll("button").find((b) => b.text() === "Télécharger")!;
    expect(downloadButtonAfter.attributes("disabled")).toBeUndefined();
  });

  it("shows an error message when generation fails", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockRejectedValueOnce("erreur de génération");
    const wrapper = mount(ReportGeneratorPage);
    const button = wrapper.findAll("button").find((b) => b.text() === "Générer")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("erreur de génération"));
  });

  it("downloads with the format the content was actually generated in, even if the selector is changed afterwards", async () => {
    URL.createObjectURL = vi.fn(() => "blob:mock-url");
    URL.revokeObjectURL = vi.fn();
    const clickSpy = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(function (this: HTMLAnchorElement) {
      capturedDownload = this.download;
    });
    let capturedDownload = "";

    const wrapper = mount(ReportGeneratorPage);
    const generateButton = wrapper.findAll("button").find((b) => b.text() === "Générer")!;
    await generateButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("generated_at"));

    // Report was generated as JSON, but the user changes the selector to
    // Markdown afterwards WITHOUT regenerating -- the downloaded file must
    // still reflect the actual (JSON) content, not the now-selected format.
    const select = wrapper.find("select");
    await select.setValue("markdown");

    const downloadButton = wrapper.findAll("button").find((b) => b.text() === "Télécharger")!;
    await downloadButton.trigger("click");

    expect(capturedDownload).toBe("nitrux-rapport.json");
    clickSpy.mockRestore();
  });
});
