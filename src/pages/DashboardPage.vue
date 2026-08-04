<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  Stethoscope, Download, RefreshCw, Wrench, FileText,
} from "lucide-vue-next";
import NxCard from "@/components/ui/NxCard.vue";
import NxStatTile from "@/components/ui/NxStatTile.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import NxQuickActionTile from "@/components/ui/NxQuickActionTile.vue";
import { usePreferencesStore } from "@/stores/preferencesStore";

interface CpuInfo { name: string; usage_percent: number; usage_display: string }
interface SystemSnapshot {
  cpus: CpuInfo[];
  memory_used_bytes: number;
  memory_total_bytes: number;
  process_count: number;
}
interface SensorSnapshot {
  battery_percent: number | null;
  battery_charging: boolean | null;
  temperatures: { label: string; celsius: number }[];
}

const emit = defineEmits<{ navigate: [string] }>();

const preferences = usePreferencesStore();
const snapshot = ref<SystemSnapshot | null>(null);
const error = ref<string | null>(null);
const sensors = ref<SensorSnapshot | null>(null);
const sensorsError = ref<string | null>(null);
let intervalId: number | undefined;

async function refresh() {
  try {
    snapshot.value = await invoke<SystemSnapshot>("get_system_snapshot");
    error.value = null;
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  }
}

async function refreshSensors() {
  try {
    sensors.value = await invoke<SensorSnapshot>("get_sensor_snapshot");
    sensorsError.value = null;
  } catch (err) {
    sensorsError.value = err instanceof Error ? err.message : String(err);
  }
}

onMounted(() => {
  refresh();
  refreshSensors();
  intervalId = window.setInterval(() => {
    refresh();
    refreshSensors();
  }, preferences.dashboardRefreshIntervalMs);
});

onUnmounted(() => {
  if (intervalId) window.clearInterval(intervalId);
});

function bytesToGb(bytes: number): string {
  return (bytes / 1024 / 1024 / 1024).toFixed(1);
}

const QUICK_ACTIONS = [
  { label: "Diagnostic", icon: Stethoscope, gradient: "linear-gradient(135deg,#f97316,#fb923c)", target: "diagnostic" },
  { label: "Installation rapide", icon: Download, gradient: "linear-gradient(135deg,#3b82f6,#2563eb)", target: "quick-install" },
  { label: "Mises à jour", icon: RefreshCw, gradient: "linear-gradient(135deg,#22c55e,#16a34a)", target: "updates" },
  { label: "Dépannage", icon: Wrench, gradient: "linear-gradient(135deg,#ef4444,#dc2626)", target: "troubleshoot" },
  { label: "Générateur de rapport", icon: FileText, gradient: "linear-gradient(135deg,#8b5cf6,#7c3aed)", target: "report-generator" },
];
</script>

<template>
  <div class="dash-page">
    <NxSectionHeader title="Vue d'ensemble" />

    <div class="dash-actions">
      <NxQuickActionTile
        v-for="action in QUICK_ACTIONS"
        :key="action.target"
        :icon="action.icon"
        :label="action.label"
        :gradient="action.gradient"
        @click="emit('navigate', action.target)"
      />
    </div>

    <NxCard v-if="error" danger>Impossible de récupérer les informations système : {{ error }}</NxCard>
    <NxCard v-if="sensorsError" danger>Impossible de récupérer les capteurs : {{ sensorsError }}</NxCard>

    <div class="dash-grid" v-if="snapshot">
      <NxCard v-for="(cpu, i) in snapshot.cpus" :key="i">
        <NxStatTile :label="cpu.name || `CPU ${i}`" :value="cpu.usage_display" />
      </NxCard>
      <NxCard>
        <NxStatTile label="Mémoire" :value="`${bytesToGb(snapshot.memory_used_bytes)} / ${bytesToGb(snapshot.memory_total_bytes)} Go`" />
      </NxCard>
      <NxCard>
        <NxStatTile label="Processus" :value="String(snapshot.process_count)" />
      </NxCard>
      <NxCard v-if="sensors?.battery_percent !== null && sensors?.battery_percent !== undefined">
        <NxStatTile label="Batterie" :value="`${sensors!.battery_percent}%${sensors!.battery_charging ? ' ⚡' : ''}`" />
      </NxCard>
      <NxCard v-for="(t, i) in sensors?.temperatures ?? []" :key="`${t.label}-${i}`">
        <NxStatTile :label="t.label" :value="`${t.celsius.toFixed(0)}°C`" />
      </NxCard>
    </div>
  </div>
</template>

<style scoped>
.dash-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.dash-actions { display: flex; gap: 12px; flex-wrap: wrap; }
.dash-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 14px; }
</style>
