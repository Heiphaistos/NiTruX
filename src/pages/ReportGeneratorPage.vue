<!-- src/pages/ReportGeneratorPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxSelect from "@/components/ui/NxSelect.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface ReportFile { filename: string; size_bytes: number; created_at: string }

type ReportFormat = "json" | "markdown" | "txt" | "html";

const FORMAT_OPTIONS: { value: ReportFormat; label: string }[] = [
  { value: "json", label: "JSON" },
  { value: "markdown", label: "Markdown" },
  { value: "txt", label: "Texte brut" },
  { value: "html", label: "HTML" },
];

const FORMAT_MIME: Record<ReportFormat, string> = {
  json: "application/json",
  markdown: "text/markdown",
  txt: "text/plain",
  html: "text/html",
};

const FORMAT_EXTENSION: Record<ReportFormat, string> = {
  json: "json",
  markdown: "md",
  txt: "txt",
  html: "html",
};

const selectedFormat = ref<ReportFormat>("json");
const generating = ref(false);
const error = ref<string | null>(null);
const reportContent = ref<string | null>(null);
// The format `reportContent` was actually generated in -- tracked
// separately from `selectedFormat` because the user can change the
// selector after generating without regenerating, and download() must
// stay faithful to the displayed content, not the current selector.
const generatedFormat = ref<ReportFormat | null>(null);

const pdfGenerating = ref(false);
const pdfError = ref<string | null>(null);

// The reports library only stores what the user actually downloads, not
// every preview -- clicking "Générer" alone must never silently write a
// file to disk.
const reports = ref<ReportFile[]>([]);
const reportsLoading = ref(false);
const reportsError = ref<string | null>(null);

async function loadReports() {
  reportsLoading.value = true;
  reportsError.value = null;
  try {
    reports.value = await invoke<ReportFile[]>("list_reports");
  } catch (e) {
    reportsError.value = String(e);
  } finally {
    reportsLoading.value = false;
  }
}

async function deleteReport(filename: string) {
  try {
    await invoke("delete_report", { filename });
    reports.value = reports.value.filter((r) => r.filename !== filename);
  } catch (e) {
    reportsError.value = String(e);
  }
}

onMounted(loadReports);

function bytesToKb(bytes: number): string {
  return (bytes / 1024).toFixed(1);
}

async function downloadPdf() {
  pdfGenerating.value = true;
  pdfError.value = null;
  try {
    const base64 = await invoke<string>("generate_pdf_report");
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    const blob = new Blob([bytes], { type: "application/pdf" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = "nitrux-rapport.pdf";
    link.click();
    URL.revokeObjectURL(url);
    const saved = await invoke<ReportFile>("save_pdf_report", { base64Content: base64 });
    reports.value = [saved, ...reports.value];
  } catch (e) {
    pdfError.value = String(e);
  } finally {
    pdfGenerating.value = false;
  }
}

async function generate() {
  generating.value = true;
  error.value = null;
  reportContent.value = null;
  try {
    reportContent.value = await invoke<string>("generate_system_report", { format: selectedFormat.value });
    generatedFormat.value = selectedFormat.value;
  } catch (e) {
    error.value = String(e);
  } finally {
    generating.value = false;
  }
}

async function download() {
  if (!reportContent.value || !generatedFormat.value) return;
  const format = generatedFormat.value;
  const content = reportContent.value;
  const blob = new Blob([content], { type: FORMAT_MIME[format] });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `nitrux-rapport.${FORMAT_EXTENSION[format]}`;
  link.click();
  URL.revokeObjectURL(url);
  try {
    const saved = await invoke<ReportFile>("save_text_report", { content, extension: FORMAT_EXTENSION[format] });
    reports.value = [saved, ...reports.value];
  } catch (e) {
    reportsError.value = String(e);
  }
}
</script>

<template>
  <div class="rg-page">
    <NxSectionHeader title="Générateur de rapport" description="Exporte un état complet du système au format de votre choix." />

    <NxCard class="rg-controls">
      <div class="rg-controls-row">
        <NxSelect v-model="selectedFormat" :options="FORMAT_OPTIONS" aria-label="Format du rapport" />
        <NxButton :disabled="generating" @click="generate">{{ generating ? "Génération..." : "Générer" }}</NxButton>
        <NxButton :disabled="!reportContent" @click="download">Télécharger</NxButton>
        <NxButton :disabled="pdfGenerating" @click="downloadPdf">{{ pdfGenerating ? "Génération PDF..." : "Télécharger en PDF" }}</NxButton>
      </div>
    </NxCard>

    <NxCard v-if="error" danger>{{ error }}</NxCard>
    <NxCard v-if="pdfError" danger>{{ pdfError }}</NxCard>

    <NxCard v-if="reportContent" class="rg-preview">
      <NxSectionHeader title="Aperçu" />
      <pre class="rg-preview-content">{{ reportContent }}</pre>
    </NxCard>

    <NxCard v-if="reportsError" danger>{{ reportsError }}</NxCard>

    <NxCard class="rg-reports">
      <NxSectionHeader :title="`Rapports générés (${reports.length})`" />
      <p v-if="reportsLoading" class="rg-reports-empty">Chargement…</p>
      <p v-else-if="reports.length === 0" class="rg-reports-empty">Aucun rapport généré pour le moment.</p>
      <div v-else class="rg-reports-list">
        <div v-for="r in reports" :key="r.filename" class="rg-report-row">
          <div class="rg-report-info">
            <span class="rg-report-name">{{ r.filename }}</span>
            <span class="rg-report-meta">{{ r.created_at }} — {{ bytesToKb(r.size_bytes) }} Ko</span>
          </div>
          <NxButton variant="danger" @click="deleteReport(r.filename)">Supprimer</NxButton>
        </div>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.rg-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.rg-controls-row { display: flex; gap: 10px; align-items: center; }
.rg-preview-content { max-height: 480px; overflow: auto; font-size: 12px; white-space: pre-wrap; word-break: break-word; margin: 0; }
.rg-reports { display: flex; flex-direction: column; gap: 10px; }
.rg-reports-empty { text-align: center; color: var(--nx-text-secondary); padding: 16px; font-size: 13px; margin: 0; }
.rg-reports-list { display: flex; flex-direction: column; gap: 4px; }
.rg-report-row { display: flex; align-items: center; gap: 10px; padding: 8px 10px; border-radius: var(--nx-style-radius); }
.rg-report-row:hover { background: var(--nx-bg-elevated); }
.rg-report-info { display: flex; flex-direction: column; gap: 2px; margin-right: auto; }
.rg-report-name { font-size: 13px; font-weight: 500; color: var(--nx-text-primary); font-family: monospace; }
.rg-report-meta { font-size: 11px; color: var(--nx-text-secondary); }
</style>
