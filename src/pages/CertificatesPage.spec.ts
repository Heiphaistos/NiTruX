import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import CertificatesPage from "./CertificatesPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_certificates") {
      return Promise.resolve([
        {
          filename: "ACCVRAIZ1.pem",
          subject: "CN = ACCVRAIZ1, OU = PKIACCV, O = ACCV, C = ES",
          issuer: "CN = ACCVRAIZ1, OU = PKIACCV, O = ACCV, C = ES",
          not_after: "Dec 31 09:37:37 2030 GMT",
          is_expired: false,
          is_expiring_soon: false,
        },
      ]);
    }
    return Promise.resolve(null);
  }),
}));

describe("CertificatesPage", () => {
  it("lists installed CA certificates with subject, issuer, and expiry date", async () => {
    const wrapper = mount(CertificatesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("ACCVRAIZ1"));
    expect(wrapper.text()).toContain("Dec 31 09:37:37 2030 GMT");
    expect(wrapper.text()).toContain("Valide");
  });

  it("filters certificates by subject", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "get_certificates") {
        return Promise.resolve([
          { filename: "a.pem", subject: "CN = Alpha CA", issuer: "CN = Alpha CA", not_after: "...", is_expired: false, is_expiring_soon: false },
          { filename: "b.pem", subject: "CN = Beta CA", issuer: "CN = Beta CA", not_after: "...", is_expired: false, is_expiring_soon: false },
        ]);
      }
      return Promise.resolve(null);
    });
    const wrapper = mount(CertificatesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Alpha CA"));
    expect(wrapper.text()).toContain("Beta CA");

    await wrapper.find(".nx-input").setValue("Alpha");
    expect(wrapper.text()).toContain("Alpha CA");
    expect(wrapper.text()).not.toContain("Beta CA");
  });

  it("shows an expired badge and a summary banner when a certificate is expired", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "get_certificates") {
        return Promise.resolve([
          { filename: "old.pem", subject: "CN = Old CA", issuer: "CN = Old CA", not_after: "Jan  1 00:00:00 2020 GMT", is_expired: true, is_expiring_soon: false },
        ]);
      }
      return Promise.resolve(null);
    });
    const wrapper = mount(CertificatesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Old CA"));
    expect(wrapper.text()).toContain("Expiré");
    expect(wrapper.text()).toContain("1 certificat(s) expiré(s)");
  });

  it("shows an expiring-soon badge distinct from expired", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "get_certificates") {
        return Promise.resolve([
          { filename: "soon.pem", subject: "CN = Soon CA", issuer: "CN = Soon CA", not_after: "...", is_expired: false, is_expiring_soon: true },
        ]);
      }
      return Promise.resolve(null);
    });
    const wrapper = mount(CertificatesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Soon CA"));
    expect(wrapper.text()).toContain("Expire bientôt");
    expect(wrapper.text()).toContain("0 certificat(s) expiré(s), 1 expirant sous 30 jours.");
  });

  it("shows an error message when the backend call fails", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "get_certificates") return Promise.reject("répertoire des certificats illisible");
      return Promise.resolve(null);
    });
    const wrapper = mount(CertificatesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("répertoire des certificats illisible"));
  });
});
