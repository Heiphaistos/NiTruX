<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  Stethoscope, Download, RefreshCw, Wrench, FileText,
} from "lucide-vue-next";
import NxCard from "@/components/ui/NxCard.vue";
import NxStatTile from "@/components/ui/NxStatTile.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import NxQuickActionTile from "@/components/ui/NxQuickActionTile.vue";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { averageCpuPercent } from "@/lib/systemMetrics";

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
interface DiskUsageEntry {
  mountpoint: string;
  total_bytes: number;
  used_bytes: number;
  used_percent: number;
}

const emit = defineEmits<{ navigate: [string] }>();

const preferences = usePreferencesStore();
const snapshot = ref<SystemSnapshot | null>(null);
const error = ref<string | null>(null);
const sensors = ref<SensorSnapshot | null>(null);
const sensorsError = ref<string | null>(null);
const diskUsage = ref<DiskUsageEntry[]>([]);
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

async function refreshDiskUsage() {
  // Best-effort, silently degrades: the health score below simply omits
  // its disk third if this never resolves (e.g. lsblk unavailable), rather
  // than surfacing yet another error card for a background metric no other
  // Dashboard tile depends on.
  try {
    diskUsage.value = await invoke<DiskUsageEntry[]>("list_disk_usage");
  } catch {
    diskUsage.value = [];
  }
}

onMounted(() => {
  refresh();
  refreshSensors();
  refreshDiskUsage();
  intervalId = window.setInterval(() => {
    refresh();
    refreshSensors();
    refreshDiskUsage();
  }, preferences.dashboardRefreshIntervalMs);
});

onUnmounted(() => {
  if (intervalId) window.clearInterval(intervalId);
});

function bytesToGb(bytes: number): string {
  return (bytes / 1024 / 1024 / 1024).toFixed(1);
}

// System health score (0-100): a NiTriTe Windows concept ported to Linux
// with the same weighting (CPU 34pts / RAM 33pts / disk 33pts, each with a
// "good"/"ok"/"poor" cutoff) rather than reinvented, since it was already a
// deliberate, working design. Uses "/" specifically, not just the first
// disk entry -- a machine with multiple mounted filesystems (data drives,
// external media) would otherwise score against whichever happened to
// sort first, not the system disk the score is actually meant to reflect.
const rootDiskPercent = computed<number | null>(() => {
  const root = diskUsage.value.find((d) => d.mountpoint === "/");
  return root ? root.used_percent : null;
});

const systemScore = computed<number | null>(() => {
  if (!snapshot.value || snapshot.value.memory_total_bytes === 0) return null;
  if (rootDiskPercent.value === null) return null;

  let score = 0;
  const cpuPct = averageCpuPercent(snapshot.value.cpus);
  if (cpuPct < 30) score += 34;
  else if (cpuPct < 60) score += 17;

  const ramFreePct = 100 - (snapshot.value.memory_used_bytes / snapshot.value.memory_total_bytes) * 100;
  if (ramFreePct > 50) score += 33;
  else if (ramFreePct > 25) score += 16;

  if (rootDiskPercent.value < 80) score += 33;
  else if (rootDiskPercent.value < 90) score += 16;

  return score;
});

function scoreLabel(score: number): string {
  if (score >= 80) return "Excellent";
  if (score >= 60) return "Bon";
  if (score >= 40) return "Moyen";
  return "Critique";
}

function scoreStatus(score: number): "success" | "warning" | "danger" {
  if (score >= 80) return "success";
  if (score >= 40) return "warning";
  return "danger";
}

// Both stops of every gradient are darkened to the same hue's darkest shade
// that still clears WCAG AA (>=4.5:1) against the tile's white text -- the
// original bright stops (e.g. #fb923c, 2.26:1) failed AA outright.
// Regenerated with the official relative-luminance formula, not eyeballed.
const QUICK_ACTIONS = [
  { label: "Diagnostic", icon: Stethoscope, gradient: "linear-gradient(135deg,#bb5604,#934403)", target: "diagnostic" },
  { label: "Installation rapide", icon: Download, gradient: "linear-gradient(135deg,#316cec,#1554e0)", target: "quick-install" },
  { label: "Mises à jour", icon: RefreshCw, gradient: "linear-gradient(135deg,#12873d,#0d632d)", target: "updates" },
  { label: "Dépannage", icon: Wrench, gradient: "linear-gradient(135deg,#dd2e2e,#c32020)", target: "troubleshoot" },
  { label: "Générateur de rapport", icon: FileText, gradient: "linear-gradient(135deg,#8a50ef,#722aec)", target: "report-generator" },
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

    <NxCard v-if="systemScore !== null" class="dash-score">
      <span class="dash-score-label">Score système</span>
      <span class="dash-score-value">{{ systemScore }}<span class="dash-score-max">/100</span></span>
      <NxBadge :status="scoreStatus(systemScore)">{{ scoreLabel(systemScore) }}</NxBadge>
    </NxCard>

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
.dash-score { display: flex; align-items: center; gap: 16px; }
.dash-score-label { font-size: 13px; color: var(--nx-text-secondary); }
.dash-score-value { font-size: 28px; font-weight: 700; font-family: var(--nx-style-font-family); margin-right: auto; }
.dash-score-max { font-size: 14px; font-weight: 400; color: var(--nx-text-secondary); }
</style>
