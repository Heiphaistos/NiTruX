<!-- src/pages/ScanPage.vue -->
<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import { buildSteps, buildReport, summarize, type ScanSection, type Severity } from "@/lib/scan";

// One click, a full A-to-Z diagnostic of the machine — hardware, sensors,
// network, certificates, crashes, antivirus, and (opt-in, one password
// prompt) the security-posture audit and firmware OEM key. Every check runs
// independently: one failure never stops the rest, and the whole thing
// exports to a single report.
const includePrivileged = ref(true);
const running = ref(false);
const sections = ref<ScanSection[]>([]);
const finishedAt = ref<Date | null>(null);
const savedName = ref<string | null>(null);
const saveError = ref<string | null>(null);

const counts = computed(() => summarize(sections.value));
const progress = computed(() => {
  const done = sections.value.filter((s) => s.status === "done" || s.status === "error" || s.status === "skipped").length;
  return sections.value.length ? Math.round((done / sections.value.length) * 100) : 0;
});

const SEVERITY_LABEL: Record<Severity, string> = { critical: "Critique", warning: "Avertissement", info: "Info", ok: "OK" };
function badgeStatus(sev: Severity): "danger" | "warning" | "info" | "success" {
  return sev === "critical" ? "danger" : sev === "warning" ? "warning" : sev === "ok" ? "success" : "info";
}

async function runScan() {
  running.value = true;
  finishedAt.value = null;
  savedName.value = null;
  saveError.value = null;
  const steps = buildSteps(includePrivileged.value);
  sections.value = steps.map((s) => ({ id: s.id, title: s.title, status: "pending", findings: [], facts: [] }));

  for (let i = 0; i < steps.length; i++) {
    sections.value[i].status = "running";
    try {
      const { findings, facts } = await steps[i].run();
      sections.value[i] = { ...sections.value[i], status: "done", findings, facts };
    } catch (e) {
      sections.value[i] = { ...sections.value[i], status: "error", error: String(e) };
    }
    // Force reactivity on the array replacement.
    sections.value = [...sections.value];
  }
  finishedAt.value = new Date();
  running.value = false;
}

function reportText(): string {
  return buildReport(sections.value, finishedAt.value ?? new Date());
}

async function saveReport() {
  saveError.value = null;
  try {
    const saved = await invoke<{ filename: string }>("save_text_report", { content: reportText(), extension: "md" });
    savedName.value = saved.filename;
  } catch (e) {
    saveError.value = String(e);
  }
}

async function copyReport() {
  try {
    await navigator.clipboard.writeText(reportText());
    savedName.value = "copié dans le presse-papiers";
  } catch {
    saveError.value = "Copie impossible dans cet environnement.";
  }
}
</script>

<template>
  <div class="sc-page">
    <NxSectionHeader
      title="Scan PC"
      description="Un diagnostic complet de la machine en un clic : matériel, capteurs, réseau, certificats, pannes, antivirus, et sécurité. Idéal avant une intervention."
    />

    <NxCard class="sc-controls">
      <label class="sc-check">
        <input type="checkbox" v-model="includePrivileged" :disabled="running" />
        Inclure l'analyse administrateur (audit de sécurité, clé OEM) — demande le mot de passe une fois
      </label>
      <NxButton :disabled="running" @click="runScan">{{ running ? "Analyse en cours..." : "Lancer le scan" }}</NxButton>
    </NxCard>

    <NxCard v-if="running || finishedAt" class="sc-summary">
      <div class="sc-progress"><div class="sc-progress-bar" :style="{ width: `${progress}%` }"></div></div>
      <div class="sc-counts">
        <NxBadge status="danger">{{ counts.critical }} critique(s)</NxBadge>
        <NxBadge status="warning">{{ counts.warning }} avertissement(s)</NxBadge>
        <NxBadge status="info">{{ counts.info }} info(s)</NxBadge>
        <NxBadge status="success">{{ counts.ok }} OK</NxBadge>
      </div>
      <div v-if="finishedAt" class="sc-actions">
        <NxButton @click="saveReport">Enregistrer le rapport (.md)</NxButton>
        <NxButton variant="ghost" @click="copyReport">Copier</NxButton>
        <NxBadge v-if="savedName" status="success" live>{{ savedName }}</NxBadge>
      </div>
      <NxCard v-if="saveError" danger>{{ saveError }}</NxCard>
    </NxCard>

    <NxCard v-for="section in sections" :key="section.id" class="sc-section">
      <div class="sc-section-head">
        <strong>{{ section.title }}</strong>
        <NxBadge v-if="section.status === 'running'" status="info">en cours…</NxBadge>
        <NxBadge v-else-if="section.status === 'error'" status="danger">erreur</NxBadge>
        <NxBadge v-else-if="section.status === 'pending'" status="info">en attente</NxBadge>
      </div>
      <NxCard v-if="section.status === 'error'" danger>{{ section.error }}</NxCard>
      <div v-else class="sc-findings">
        <div v-for="(finding, i) in section.findings" :key="i" class="sc-finding">
          <NxBadge :status="badgeStatus(finding.severity)">{{ SEVERITY_LABEL[finding.severity] }}</NxBadge>
          <div class="sc-finding-body">
            <span class="sc-finding-title">{{ finding.title }}</span>
            <span v-if="finding.detail" class="sc-finding-detail">{{ finding.detail }}</span>
          </div>
        </div>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.sc-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.sc-controls { display: flex; align-items: center; justify-content: space-between; gap: 16px; flex-wrap: wrap; }
.sc-check { display: flex; align-items: center; gap: 8px; font-size: 13px; }
.sc-summary { display: flex; flex-direction: column; gap: 10px; }
.sc-progress { width: 100%; height: 6px; border-radius: 3px; background: var(--nx-bg-elevated); overflow: hidden; }
.sc-progress-bar { height: 100%; background: var(--nx-accent-primary); transition: width 0.3s ease; }
.sc-counts { display: flex; gap: 8px; flex-wrap: wrap; }
.sc-actions { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.sc-section { display: flex; flex-direction: column; gap: 8px; }
.sc-section-head { display: flex; align-items: center; gap: 10px; }
.sc-findings { display: flex; flex-direction: column; gap: 6px; }
.sc-finding { display: flex; gap: 10px; align-items: flex-start; }
.sc-finding-body { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.sc-finding-title { font-size: 13px; font-weight: 500; }
.sc-finding-detail { font-size: 12px; color: var(--nx-text-secondary); white-space: pre-wrap; word-break: break-word; }
</style>
