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
    if (cmd === "generate_pdf_report") {
      // base64 of the literal bytes "%PDF-1.7 fake" -- just needs to be
      // valid base64 that round-trips through atob, not a real PDF, since
      // this test only proves the download plumbing (Blob/anchor), not
      // printpdf's rendering (already covered by report.rs's own Rust tests).
      return Promise.resolve(btoa("%PDF-1.7 fake"));
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

  it("downloads a PDF via generate_pdf_report, independently of the text-format selector/preview state", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    URL.createObjectURL = vi.fn(() => "blob:mock-pdf-url");
    URL.revokeObjectURL = vi.fn();
    let capturedDownload = "";
    let capturedBlob: Blob | undefined;
    const originalCreateObjectURL = URL.createObjectURL;
    URL.createObjectURL = vi.fn((blob: Blob) => {
      capturedBlob = blob;
      return originalCreateObjectURL(blob);
    });
    const clickSpy = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(function (this: HTMLAnchorElement) {
      capturedDownload = this.download;
    });

    // No prior "Générer" click -- PDF export must work standalone.
    const wrapper = mount(ReportGeneratorPage);
    const pdfButton = wrapper.findAll("button").find((b) => b.text() === "Télécharger en PDF")!;
    await pdfButton.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("generate_pdf_report"));
    await vi.waitFor(() => expect(capturedDownload).toBe("nitrux-rapport.pdf"));

    expect(capturedBlob?.type).toBe("application/pdf");
    clickSpy.mockRestore();
  });

  it("shows an error message when PDF generation fails", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "generate_pdf_report") return Promise.reject("échec de génération du PDF : police introuvable");
      return Promise.resolve(null);
    });
    const wrapper = mount(ReportGeneratorPage);
    const pdfButton = wrapper.findAll("button").find((b) => b.text() === "Télécharger en PDF")!;
    await pdfButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("police introuvable"));
  });
});
