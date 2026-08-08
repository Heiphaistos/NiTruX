// src/pages/ReportGeneratorPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import ReportGeneratorPage from "./ReportGeneratorPage.vue";

// `vi.hoisted` so this is reachable both from the hoisted `vi.mock` factory
// below AND from `beforeEach` -- without re-applying it every test, a test
// further down the file that calls `mockImplementation(...)` (not the
// `Once` variant) permanently replaces this for every test that runs after
// it, since `vi.clearAllMocks()` clears call history but not the
// implementation. Same contamination bug class already found and fixed in
// DashboardPage.spec.ts.
const defaultInvokeImpl = vi.hoisted(() => (cmd: string, args?: Record<string, unknown>) => {
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
  if (cmd === "list_reports") return Promise.resolve([]);
  if (cmd === "save_text_report") {
    return Promise.resolve({ filename: `rapport-999.${args?.extension}`, size_bytes: 10, created_at: "2026-01-01T00:00:00Z" });
  }
  if (cmd === "save_pdf_report") {
    return Promise.resolve({ filename: "rapport-999.pdf", size_bytes: 20, created_at: "2026-01-01T00:00:00Z" });
  }
  if (cmd === "delete_report") return Promise.resolve(null);
  return Promise.resolve(null);
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(defaultInvokeImpl),
}));

describe("ReportGeneratorPage", () => {
  beforeEach(async () => {
    vi.clearAllMocks();
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation(defaultInvokeImpl);
  });

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
    // Command-aware, not a blind mockRejectedValueOnce -- onMounted now
    // also calls list_reports on every mount, and a bare "reject the very
    // next call" would race against that unrelated call instead of the
    // "Générer" click this test actually means to fail.
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "generate_system_report") return Promise.reject("erreur de génération");
      return defaultInvokeImpl(cmd, args);
    });
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
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "generate_pdf_report") return Promise.reject("échec de génération du PDF : police introuvable");
      return defaultInvokeImpl(cmd, args);
    });
    const wrapper = mount(ReportGeneratorPage);
    const pdfButton = wrapper.findAll("button").find((b) => b.text() === "Télécharger en PDF")!;
    await pdfButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("police introuvable"));
  });

  it("loads and displays previously saved reports on mount", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "list_reports") {
        return Promise.resolve([{ filename: "rapport-100.json", size_bytes: 2048, created_at: "2026-01-01T10:00:00Z" }]);
      }
      return Promise.resolve(null);
    });
    const wrapper = mount(ReportGeneratorPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("rapport-100.json"));
    expect(wrapper.text()).toContain("2.0 Ko");
  });

  it("shows the empty state when no report has ever been saved", async () => {
    const wrapper = mount(ReportGeneratorPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucun rapport généré pour le moment."));
  });

  it("clicking Télécharger persists the generated report and it appears in the list", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(ReportGeneratorPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucun rapport généré pour le moment."));

    const generateButton = wrapper.findAll("button").find((b) => b.text() === "Générer")!;
    await generateButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("generated_at"));

    const downloadButton = wrapper.findAll("button").find((b) => b.text() === "Télécharger")!;
    await downloadButton.trigger("click");

    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("save_text_report", { content: '{"generated_at":"epoch:123"}', extension: "json" }));
    await vi.waitFor(() => expect(wrapper.text()).toContain("rapport-999.json"));
  });

  it("clicking Télécharger en PDF persists the PDF report too", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(ReportGeneratorPage);
    const pdfButton = wrapper.findAll("button").find((b) => b.text() === "Télécharger en PDF")!;
    await pdfButton.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("save_pdf_report", { base64Content: btoa("%PDF-1.7 fake") }));
    await vi.waitFor(() => expect(wrapper.text()).toContain("rapport-999.pdf"));
  });

  it("deletes a report from the list when Supprimer is clicked", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "list_reports") {
        return Promise.resolve([{ filename: "rapport-100.json", size_bytes: 2048, created_at: "2026-01-01T10:00:00Z" }]);
      }
      if (cmd === "delete_report") return Promise.resolve(null);
      return Promise.resolve(null);
    });
    const wrapper = mount(ReportGeneratorPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("rapport-100.json"));

    const deleteButton = wrapper.findAll("button").find((b) => b.text() === "Supprimer")!;
    await deleteButton.trigger("click");

    expect(invoke).toHaveBeenCalledWith("delete_report", { filename: "rapport-100.json" });
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucun rapport généré pour le moment."));
  });

  it("shows an error message when the reports list fails to load", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "list_reports") return Promise.reject("répertoire des rapports illisible");
      return Promise.resolve(null);
    });
    const wrapper = mount(ReportGeneratorPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("répertoire des rapports illisible"));
  });
});
