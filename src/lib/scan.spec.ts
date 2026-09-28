import { describe, it, expect, vi, beforeEach } from "vitest";
import { buildSteps, buildReport, summarize, type ScanSection } from "./scan";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("scan orchestration", () => {
  beforeEach(() => {
    invoke.mockReset();
    invoke.mockImplementation((cmd: string) => {
      switch (cmd) {
        case "get_hardware_inventory": return Promise.resolve({
          cpu: { title: "Processeur", items: [{ key: "Modèle", value: "CPU" }, { key: "Virtualisation matérielle", value: "disponible" }, { key: "Failles CPU non corrigées", value: "mmio_stale_data" }] },
          firmware: { title: "Firmware", items: [{ key: "TPM", value: "absent ou désactivé dans le BIOS" }, { key: "Secure Boot", value: "désactivé" }] },
          memory: { title: "Mémoire", items: [] }, gpus: [], storage: [], network: [],
        });
        case "get_all_sensors": return Promise.resolve([
          { category: "cpu", device: "CPU", readings: [{ kind: "temperature", label: "Pkg", value: 95, unit: "°C", critical: 90, alarm: false }] },
        ]);
        case "get_network_snapshot": return Promise.resolve({ listening_ports: [{ port: 22, process: "sshd", protocol: "tcp" }], dns_servers: ["8.8.8.8"], routes: [{ destination: "default", gateway: "192.168.1.1" }], wifi_networks: [] });
        case "get_certificates": return Promise.resolve([{ subject: "old", not_after: "2020", is_expired: true, is_expiring_soon: false }]);
        case "get_crash_events": return Promise.resolve([]);
        case "get_environment_variables": return Promise.resolve([["HOME", "/home/tester"]]);
        case "scan_for_malware": return Promise.resolve([]);
        case "check_required_tools": return Promise.resolve([{ binary: "smartctl", installed: false, feature: "SMART" }]);
        case "run_security_audit": return Promise.resolve([{ severity: "critical", title: "SSH root", detail: "PermitRootLogin yes" }]);
        case "get_firmware_windows_key": return Promise.resolve("XXXXX-YYYYY-ZZZZZ-AAAAA-BBBBB");
        default: return Promise.resolve(null);
      }
    });
  });

  async function runAll(privileged: boolean): Promise<ScanSection[]> {
    const steps = buildSteps(privileged);
    const sections: ScanSection[] = [];
    for (const step of steps) {
      const { findings, facts } = await step.run();
      sections.push({ id: step.id, title: step.title, status: "done", findings, facts });
    }
    return sections;
  }

  it("omits the privileged steps unless asked", () => {
    expect(buildSteps(false).some((s) => s.privileged)).toBe(false);
    expect(buildSteps(true).filter((s) => s.privileged).map((s) => s.id)).toEqual(["security", "firmware-key"]);
  });

  it("grades real conditions across every section", async () => {
    const sections = await runAll(true);
    const counts = summarize(sections);
    expect(counts.critical).toBeGreaterThanOrEqual(1); // SSH root
    expect(counts.warning).toBeGreaterThanOrEqual(1); // TPM absent / hot CPU / expired cert
    const titles = sections.flatMap((s) => s.findings.map((f) => f.title));
    expect(titles).toContain("TPM absent ou désactivé");
    expect(titles).toContain("Failles CPU non corrigées");
    expect(titles.some((t) => t.includes("Température élevée"))).toBe(true);
    expect(titles.some((t) => t.includes("expiré"))).toBe(true);
    expect(titles).toContain("SSH root");
  });

  it("assembles a markdown report with a summary and section headers", async () => {
    const sections = await runAll(false);
    const report = buildReport(sections, new Date("2026-09-28T10:00:00Z"));
    expect(report).toContain("# Rapport de diagnostic NiTruX");
    expect(report).toContain("## Matériel & firmware");
    expect(report).toMatch(/Résumé : \d+ critique/);
    expect(report).toContain("🟠");
  });
});
