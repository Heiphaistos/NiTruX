// "Scan PC": runs the whole diagnostic in sequence and assembles one
// graded report. The unprivileged half always runs; the administrator half
// (security posture, firmware key, RAM detail) runs only when asked, behind
// one polkit prompt. Everything here is orchestration over commands the
// backend already exposes — no new privilege surface.
import { invoke } from "@tauri-apps/api/core";

export type Severity = "critical" | "warning" | "info" | "ok";

export interface ScanFinding {
  severity: Severity;
  title: string;
  detail: string;
}

export interface ScanSection {
  id: string;
  title: string;
  status: "pending" | "running" | "done" | "skipped" | "error";
  findings: ScanFinding[];
  /** Raw key/value lines for the exported report. */
  facts: { key: string; value: string }[];
  error?: string;
}

export interface ScanStep {
  id: string;
  title: string;
  privileged: boolean;
  run: () => Promise<{ findings: ScanFinding[]; facts: { key: string; value: string }[] }>;
}

const f = (severity: Severity, title: string, detail = ""): ScanFinding => ({ severity, title, detail });
const kv = (key: string, value: string) => ({ key, value });

interface KeyValue { key: string; value: string }
interface Section { title: string; items: KeyValue[] }
interface Inventory {
  cpu: Section; firmware: Section; memory: Section;
  gpus: { name: string }[]; storage: { name: string; details: KeyValue[] }[];
  network: { name: string }[];
}
interface Chip { category: string; device: string; readings: { kind: string; label: string; value: number; unit: string; critical: number | null; alarm: boolean }[] }
interface NetworkSnapshot { listening_ports: { port: number; process: string | null; protocol: string | null }[]; dns_servers: string[]; routes: { destination: string; gateway: string | null }[]; wifi_networks: unknown[] }
interface CertEntry { subject: string; not_after: string; is_expired: boolean; is_expiring_soon: boolean }
interface CrashEvent { kind: string; message: string; unit: string }
interface ToolStatus { binary: string; installed: boolean; feature: string }
interface SecurityFinding { severity: string; title: string; detail: string }
interface Malware { path: string; signature: string }

function firmwareItem(inv: Inventory, key: string): string | undefined {
  return inv.firmware.items.find((i) => i.key === key)?.value;
}

export function buildSteps(includePrivileged: boolean): ScanStep[] {
  const steps: ScanStep[] = [
    {
      id: "hardware",
      title: "Matériel & firmware",
      privileged: false,
      run: async () => {
        const inv = await invoke<Inventory>("get_hardware_inventory");
        const facts = [
          ...inv.cpu.items.map((i) => kv(`CPU · ${i.key}`, i.value)),
          ...inv.firmware.items.map((i) => kv(`Firmware · ${i.key}`, i.value)),
          ...inv.memory.items.map((i) => kv(`Mémoire · ${i.key}`, i.value)),
          kv("GPU", inv.gpus.map((g) => g.name).join(", ") || "aucun"),
          kv("Stockage", inv.storage.map((s) => s.name).join(", ") || "aucun"),
        ];
        const findings: ScanFinding[] = [];
        const tpm = firmwareItem(inv, "TPM") ?? "";
        findings.push(tpm.startsWith("présent") ? f("ok", "TPM présent", tpm) : f("warning", "TPM absent ou désactivé", tpm));
        const sb = firmwareItem(inv, "Secure Boot");
        if (sb) findings.push(sb === "activé" ? f("ok", "Secure Boot activé") : f("warning", "Secure Boot désactivé", sb));
        const virt = inv.cpu.items.find((i) => i.key === "Virtualisation matérielle")?.value ?? "";
        if (virt && !virt.startsWith("disponible")) findings.push(f("info", "Virtualisation matérielle indisponible", virt));
        const vulns = inv.cpu.items.find((i) => i.key === "Failles CPU non corrigées")?.value;
        if (vulns && vulns !== "aucune") findings.push(f("warning", "Failles CPU non corrigées", vulns));
        return { findings, facts };
      },
    },
    {
      id: "sensors",
      title: "Températures & capteurs",
      privileged: false,
      run: async () => {
        const chips = await invoke<Chip[]>("get_all_sensors");
        const facts: KeyValue[] = [];
        const findings: ScanFinding[] = [];
        let hot = 0;
        for (const c of chips) {
          for (const r of c.readings) {
            facts.push(kv(`${c.device || c.category} · ${r.label}`, `${r.value} ${r.unit}`));
            if (r.kind === "temperature" && (r.alarm || (r.critical && r.value >= r.critical))) {
              hot++;
              findings.push(f("warning", `Température élevée : ${c.device || c.category}`, `${r.label} à ${r.value} ${r.unit}`));
            }
          }
        }
        if (chips.length === 0) findings.push(f("info", "Aucun capteur détecté"));
        else if (hot === 0) findings.push(f("ok", `${chips.length} puce(s) de capteurs, aucune surchauffe`));
        return { findings, facts };
      },
    },
    {
      id: "network",
      title: "Réseau",
      privileged: false,
      run: async () => {
        const net = await invoke<NetworkSnapshot>("get_network_snapshot");
        const facts = [
          kv("Serveurs DNS", net.dns_servers.join(", ") || "aucun"),
          kv("Passerelle(s)", net.routes.filter((r) => r.destination === "default" || r.destination === "0.0.0.0").map((r) => r.gateway ?? "?").join(", ") || "aucune"),
          kv("Réseaux Wi-Fi visibles", String(net.wifi_networks.length)),
          ...net.listening_ports.map((p) => kv(`Port en écoute ${p.protocol ?? "?"}/${p.port}`, p.process ?? "?")),
        ];
        const findings: ScanFinding[] = [];
        const exposed = net.listening_ports.filter((p) => (p.protocol ?? "").toLowerCase().includes("tcp"));
        findings.push(exposed.length === 0
          ? f("ok", "Aucun port TCP en écoute")
          : f("info", `${exposed.length} port(s) TCP en écoute`, exposed.map((p) => `${p.port} (${p.process || "?"})`).join(", ")));
        return { findings, facts };
      },
    },
    {
      id: "certificates",
      title: "Certificats",
      privileged: false,
      run: async () => {
        const certs = await invoke<CertEntry[]>("get_certificates");
        const expired = certs.filter((c) => c.is_expired);
        const soon = certs.filter((c) => c.is_expiring_soon && !c.is_expired);
        const findings: ScanFinding[] = [];
        if (expired.length) findings.push(f("warning", `${expired.length} certificat(s) expiré(s)`, expired.slice(0, 10).map((c) => c.subject).join("\n")));
        if (soon.length) findings.push(f("info", `${soon.length} certificat(s) expirant bientôt`));
        if (!expired.length && !soon.length) findings.push(f("ok", `${certs.length} certificat(s), aucun expiré`));
        return { findings, facts: [kv("Certificats installés", String(certs.length))] };
      },
    },
    {
      id: "crashes",
      title: "Pannes récentes",
      privileged: false,
      run: async () => {
        const events = await invoke<CrashEvent[]>("get_crash_events");
        const findings = events.length
          ? [f("warning", `${events.length} panne(s) dans les journaux`, events.slice(0, 8).map((e) => `${e.kind} · ${e.unit}: ${e.message}`).join("\n"))]
          : [f("ok", "Aucune panne récente (kernel panic, OOM, segfault)")];
        return { findings, facts: [kv("Pannes détectées", String(events.length))] };
      },
    },
    {
      id: "antivirus",
      title: "Antivirus (ClamAV)",
      privileged: false,
      run: async () => {
        try {
          // clamscan needs a real path (no ~ expansion). Scan the user's
          // home — the command carries its own generous timeout backend-side.
          const env = await invoke<[string, string][]>("get_environment_variables");
          const home = env.find(([k]) => k === "HOME")?.[1];
          if (!home) return { findings: [f("info", "Antivirus ignoré", "dossier personnel introuvable")], facts: [] };
          const hits = await invoke<Malware[]>("scan_for_malware", { directory: home });
          return {
            findings: hits.length
              ? [f("critical", `${hits.length} menace(s) détectée(s)`, hits.slice(0, 20).map((m) => `${m.signature} — ${m.path}`).join("\n"))]
              : [f("ok", "Aucune menace détectée dans le dossier personnel")],
            facts: [kv("Menaces détectées", String(hits.length))],
          };
        } catch (e) {
          return { findings: [f("info", "Antivirus indisponible", String(e))], facts: [] };
        }
      },
    },
    {
      id: "tools",
      title: "Outils système",
      privileged: false,
      run: async () => {
        const tools = await invoke<ToolStatus[]>("check_required_tools");
        const missing = tools.filter((t) => !t.installed);
        return {
          findings: missing.length
            ? [f("info", `${missing.length} outil(s) système absent(s)`, missing.map((t) => `${t.binary} → ${t.feature}`).join("\n"))]
            : [f("ok", "Tous les outils système sont présents")],
          facts: tools.map((t) => kv(t.binary, t.installed ? "installé" : "absent")),
        };
      },
    },
  ];

  if (includePrivileged) {
    steps.push(
      {
        id: "security",
        title: "Audit de sécurité (admin)",
        privileged: true,
        run: async () => {
          const raw = await invoke<SecurityFinding[]>("run_security_audit");
          const findings = raw.map((r) => f((r.severity as Severity) ?? "info", r.title, r.detail));
          return { findings, facts: [] };
        },
      },
      {
        id: "firmware-key",
        title: "Clé Windows OEM (admin)",
        privileged: true,
        run: async () => {
          const key = await invoke<string | null>("get_firmware_windows_key");
          return {
            findings: [key ? f("info", "Clé Windows OEM présente dans le firmware", key) : f("ok", "Aucune clé OEM dans le firmware")],
            facts: key ? [kv("Clé Windows OEM", key)] : [],
          };
        },
      },
    );
  }
  return steps;
}

const RANK: Record<Severity, number> = { critical: 0, warning: 1, info: 2, ok: 3 };

export function allFindings(sections: ScanSection[]): ScanFinding[] {
  return sections.flatMap((s) => s.findings).sort((a, b) => RANK[a.severity] - RANK[b.severity]);
}

export function summarize(sections: ScanSection[]): Record<Severity, number> {
  const counts: Record<Severity, number> = { critical: 0, warning: 0, info: 0, ok: 0 };
  for (const finding of sections.flatMap((s) => s.findings)) counts[finding.severity]++;
  return counts;
}

export function buildReport(sections: ScanSection[], when: Date): string {
  const lines: string[] = [`# Rapport de diagnostic NiTruX`, "", `Généré le ${when.toLocaleString("fr-FR")}`, ""];
  const counts = summarize(sections);
  lines.push(`Résumé : ${counts.critical} critique(s), ${counts.warning} avertissement(s), ${counts.info} info(s), ${counts.ok} OK.`, "");
  for (const section of sections) {
    lines.push(`## ${section.title}`, "");
    if (section.status === "error") {
      lines.push(`_Erreur : ${section.error}_`, "");
      continue;
    }
    if (section.status === "skipped") {
      lines.push("_Ignoré._", "");
      continue;
    }
    for (const finding of section.findings) {
      const mark = finding.severity === "critical" ? "🔴" : finding.severity === "warning" ? "🟠" : finding.severity === "ok" ? "🟢" : "🔵";
      lines.push(`- ${mark} **${finding.title}**${finding.detail ? ` — ${finding.detail.replace(/\n/g, "; ")}` : ""}`);
    }
    if (section.facts.length) {
      lines.push("", "<details><summary>Détails</summary>", "");
      for (const fact of section.facts) lines.push(`- ${fact.key} : ${fact.value}`);
      lines.push("", "</details>");
    }
    lines.push("");
  }
  return lines.join("\n");
}
