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

const MAX_SAMPLES = 60;

const preferences = usePreferencesStore();
const samples = ref<PerfSample[]>([]);
const cpuHistory = computed(() => samples.value.map((s) => s.cpuPercent));
const memoryHistory = computed(() => samples.value.map((s) => s.memoryPercent));
const error = ref<string | null>(null);
let intervalId: number | undefined;

async function sample() {
  try {
    const snapshot = await invoke<SystemSnapshot>("get_system_snapshot");
    samples.value.push({
      timestamp: Date.now(),
      cpuPercent: averageCpuPercent(snapshot.cpus),
      memoryPercent: memoryUsedPercent(snapshot.memory_used_bytes, snapshot.memory_total_bytes),
    });
    if (samples.value.length > MAX_SAMPLES) samples.value.shift();
    error.value = null;
  } catch (e) {
    error.value = String(e);
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

onMounted(() => {
  sample();
  intervalId = window.setInterval(sample, preferences.dashboardRefreshIntervalMs);
});

onUnmounted(() => {
  if (intervalId) window.clearInterval(intervalId);
});
</script>

<template>
  <div class="perf-page">
    <NxSectionHeader title="Historique perf." description="CPU et mémoire depuis l'ouverture de cette page (non persisté au-delà de cette session -- exportez en CSV pour garder une trace)." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <NxCard>
      <NxSectionHeader title="CPU (%)" />
      <NxSparkline :values="cpuHistory" :width="600" :height="80" />
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Mémoire (%)" />
      <NxSparkline :values="memoryHistory" :width="600" :height="80" />
    </NxCard>

    <NxButton :disabled="samples.length === 0" @click="exportCsv">Exporter en CSV</NxButton>
  </div>
</template>

<style scoped>
.perf-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
</style>
