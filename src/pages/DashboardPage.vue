<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

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
  }, 2000);
});

onUnmounted(() => {
  if (intervalId) window.clearInterval(intervalId);
});

function bytesToGb(bytes: number): string {
  return (bytes / 1024 / 1024 / 1024).toFixed(1);
}
</script>

<template>
  <div class="dash-page">
    <h1>Vue d'ensemble</h1>
    <div class="dash-error" v-if="error">
      Impossible de récupérer les informations système : {{ error }}
    </div>
    <div class="dash-error" v-if="sensorsError">
      Impossible de récupérer les capteurs : {{ sensorsError }}
    </div>
    <div class="dash-grid" v-if="snapshot">
      <div class="dash-card" v-for="(cpu, i) in snapshot.cpus" :key="i">
        <div class="dash-label">{{ cpu.name || `CPU ${i}` }}</div>
        <div class="dash-value">{{ cpu.usage_display }}</div>
      </div>
      <div class="dash-card">
        <div class="dash-label">Mémoire</div>
        <div class="dash-value">{{ bytesToGb(snapshot.memory_used_bytes) }} / {{ bytesToGb(snapshot.memory_total_bytes) }} GB</div>
      </div>
      <div class="dash-card">
        <div class="dash-label">Processus</div>
        <div class="dash-value">{{ snapshot.process_count }}</div>
      </div>
      <div class="dash-card" v-if="sensors?.battery_percent !== null && sensors?.battery_percent !== undefined">
        <div class="dash-label">Batterie</div>
        <div class="dash-value">{{ sensors!.battery_percent }}%{{ sensors!.battery_charging ? " ⚡" : "" }}</div>
      </div>
      <div class="dash-card" v-for="(t, i) in sensors?.temperatures ?? []" :key="`${t.label}-${i}`">
        <div class="dash-label">{{ t.label }}</div>
        <div class="dash-value">{{ t.celsius.toFixed(0) }}°C</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dash-page { padding: 24px; color: var(--nx-text-primary); }
.dash-error { margin-top: 16px; padding: 12px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-secondary); }
.dash-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 14px; margin-top: 16px; }
.dash-card { background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); border-radius: 10px; padding: 14px; }
.dash-label { font-size: 12px; color: var(--nx-text-secondary); }
.dash-value { font-size: 22px; font-weight: 700; margin-top: 6px; }
</style>
