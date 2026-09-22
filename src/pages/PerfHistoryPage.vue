<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import NxSparkline from "@/components/ui/NxSparkline.vue";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { averageCpuPercent, memoryUsedPercent } from "@/lib/systemMetrics";
import { buildPerfHistoryCsv, type PerfSample } from "@/lib/perfHistoryCsv";

interface CpuInfo { usage_percent: number }
interface SystemSnapshot { cpus: CpuInfo[]; memory_used_bytes: number; memory_total_bytes: number }

interface StoredSample { timestamp_ms: number; cpu_percent: number; memory_percent: number }

/** How many past samples the graphs show. The file keeps far more. */
const MAX_SAMPLES = 60;

const preferences = usePreferencesStore();
const samples = ref<PerfSample[]>([]);
const historyError = ref<string | null>(null);
const clearing = ref(false);
const cpuHistory = computed(() => samples.value.map((s) => s.cpuPercent));
const memoryHistory = computed(() => samples.value.map((s) => s.memoryPercent));
const error = ref<string | null>(null);
let intervalId: number | undefined;

async function sample() {
  try {
    const snapshot = await invoke<SystemSnapshot>("get_system_snapshot");
    const cpuPercent = averageCpuPercent(snapshot.cpus);
    const memoryPercent = memoryUsedPercent(snapshot.memory_used_bytes, snapshot.memory_total_bytes);
    samples.value.push({ timestamp: Date.now(), cpuPercent, memoryPercent });
    if (samples.value.length > MAX_SAMPLES) samples.value.shift();
    error.value = null;
    // Persisting is a separate concern from sampling: a failing write must
    // not blank the live graphs that just updated correctly.
    try {
      await invoke("record_perf_sample", { cpuPercent, memoryPercent });
      historyError.value = null;
    } catch (e) {
      historyError.value = String(e);
    }
  } catch (e) {
    error.value = String(e);
  }
}

async function loadHistory() {
  try {
    const stored = await invoke<StoredSample[]>("get_perf_history");
    samples.value = stored.slice(-MAX_SAMPLES).map((s) => ({
      timestamp: s.timestamp_ms,
      cpuPercent: s.cpu_percent,
      memoryPercent: s.memory_percent,
    }));
  } catch (e) {
    historyError.value = String(e);
  }
}

async function clearHistory() {
  clearing.value = true;
  try {
    await invoke("clear_perf_history");
    samples.value = [];
    historyError.value = null;
  } catch (e) {
    historyError.value = String(e);
  } finally {
    clearing.value = false;
  }
}

function exportCsv() {
  const csv = buildPerfHistoryCsv(samples.value);
  const blob = new Blob([csv], { type: "text/csv" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `historique-perf_${new Date().toISOString().replace(/[:.]/g, "-")}.csv`;
  a.click();
  URL.revokeObjectURL(url);
}

onMounted(async () => {
  // Past samples first, so the graphs open with history rather than one
  // point, then keep sampling on top of them.
  await loadHistory();
  sample();
  intervalId = window.setInterval(sample, preferences.dashboardRefreshIntervalMs);
});

onUnmounted(() => {
  if (intervalId) window.clearInterval(intervalId);
});
</script>

<template>
  <div class="perf-page">
    <NxSectionHeader title="Historique perf." description="CPU et mémoire, conservés sur disque entre les sessions (environ 24 h de mesures)." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>
    <NxCard v-if="historyError" danger>Historique non enregistré : {{ historyError }}</NxCard>

    <NxCard>
      <NxSectionHeader title="CPU (%)" />
      <NxSparkline :values="cpuHistory" :width="600" :height="80" />
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Mémoire (%)" />
      <NxSparkline :values="memoryHistory" :width="600" :height="80" />
    </NxCard>

    <div class="perf-actions">
      <NxButton :disabled="samples.length === 0" @click="exportCsv">Exporter en CSV</NxButton>
      <NxButton variant="danger" :disabled="clearing || samples.length === 0" @click="clearHistory">
        {{ clearing ? "Effacement..." : "Effacer l'historique" }}
      </NxButton>
    </div>
  </div>
</template>

<style scoped>
.perf-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.perf-actions { display: flex; gap: 10px; flex-wrap: wrap; }
</style>
