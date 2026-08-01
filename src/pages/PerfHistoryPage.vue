<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import NxSparkline from "@/components/ui/NxSparkline.vue";
import { usePreferencesStore } from "@/stores/preferencesStore";

interface CpuInfo { usage_percent: number }
interface SystemSnapshot { cpus: CpuInfo[]; memory_used_bytes: number; memory_total_bytes: number }

const MAX_SAMPLES = 60;

const preferences = usePreferencesStore();
const cpuHistory = ref<number[]>([]);
const memoryHistory = ref<number[]>([]);
const error = ref<string | null>(null);
let intervalId: number | undefined;

function averageCpuPercent(cpus: CpuInfo[]): number {
  if (cpus.length === 0) return 0;
  return cpus.reduce((sum, c) => sum + c.usage_percent, 0) / cpus.length;
}

function pushSample(arr: typeof cpuHistory, value: number) {
  arr.value.push(value);
  if (arr.value.length > MAX_SAMPLES) arr.value.shift();
}

async function sample() {
  try {
    const snapshot = await invoke<SystemSnapshot>("get_system_snapshot");
    pushSample(cpuHistory, averageCpuPercent(snapshot.cpus));
    pushSample(memoryHistory, (snapshot.memory_used_bytes / snapshot.memory_total_bytes) * 100);
    error.value = null;
  } catch (e) {
    error.value = String(e);
  }
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
    <NxSectionHeader title="Historique perf." description="CPU et mémoire depuis l'ouverture de cette page (non persisté)." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <NxCard>
      <NxSectionHeader title="CPU (%)" />
      <NxSparkline :values="cpuHistory" :width="600" :height="80" />
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Mémoire (%)" />
      <NxSparkline :values="memoryHistory" :width="600" :height="80" />
    </NxCard>
  </div>
</template>

<style scoped>
.perf-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
</style>
