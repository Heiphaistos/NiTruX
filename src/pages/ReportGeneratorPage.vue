<!-- src/pages/ReportGeneratorPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxSelect from "@/components/ui/NxSelect.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

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

function download() {
  if (!reportContent.value || !generatedFormat.value) return;
  const blob = new Blob([reportContent.value], { type: FORMAT_MIME[generatedFormat.value] });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `nitrux-rapport.${FORMAT_EXTENSION[generatedFormat.value]}`;
  link.click();
  URL.revokeObjectURL(url);
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
  </div>
</template>

<style scoped>
.rg-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.rg-controls-row { display: flex; gap: 10px; align-items: center; }
.rg-preview-content { max-height: 480px; overflow: auto; font-size: 12px; white-space: pre-wrap; word-break: break-word; margin: 0; }
</style>
