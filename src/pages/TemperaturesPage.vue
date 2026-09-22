<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxStatTile from "@/components/ui/NxStatTile.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface TemperatureReading { label: string; celsius: number }
interface SensorSnapshot { battery_percent: number | null; battery_charging: boolean | null; temperatures: TemperatureReading[] }

interface GpuStatus {
  name: string;
  temperature_celsius: number | null;
  utilization_percent: number | null;
  memory_used_mb: number | null;
  memory_total_mb: number | null;
}
interface GpuSnapshot { nvidia_available: boolean; gpus: GpuStatus[] }

const snapshot = ref<SensorSnapshot | null>(null);
const error = ref<string | null>(null);
const gpu = ref<GpuSnapshot | null>(null);

onMounted(async () => {
  try {
    snapshot.value = await invoke<SensorSnapshot>("get_sensor_snapshot");
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  }
  // Separate call, separate failure: hwmon sensors and an NVIDIA card are
  // unrelated sources, and most machines have no nvidia-smi at all.
  try {
    gpu.value = await invoke<GpuSnapshot>("get_gpu_snapshot");
  } catch {
    gpu.value = null;
  }
});

function statusFor(celsius: number): "success" | "warning" | "danger" {
  if (celsius > 80) return "danger";
  if (celsius >= 60) return "warning";
  return "success";
}
</script>

<template>
  <div class="temp-page">
    <NxSectionHeader title="Températures" description="Relevés des capteurs thermiques détectés sur le système." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <div v-else-if="snapshot && snapshot.temperatures.length === 0" class="temp-empty">
      Aucun capteur de température détecté sur ce système.
    </div>

    <div class="temp-grid" v-else-if="snapshot">
      <NxCard v-for="(t, i) in snapshot.temperatures" :key="`${t.label}-${i}`">
        <div class="temp-card-inner">
          <NxStatTile :label="t.label" :value="`${t.celsius.toFixed(0)}°C`" />
          <NxBadge :status="statusFor(t.celsius)">
            {{ statusFor(t.celsius) === "danger" ? "élevé" : statusFor(t.celsius) === "warning" ? "modéré" : "normal" }}
          </NxBadge>
        </div>
      </NxCard>
    </div>

    <NxCard v-if="gpu">
      <NxSectionHeader title="Carte graphique NVIDIA" />
      <div v-if="!gpu.nvidia_available" class="temp-empty">
        Aucune carte NVIDIA détectée (les GPU AMD et Intel remontent leurs températures dans la liste ci-dessus).
      </div>
      <div v-for="(g, gi) in gpu.gpus" :key="`${g.name}-${gi}`" class="temp-gpu-row">
        <span>{{ g.name }}</span>
        <span>
          <template v-if="g.temperature_celsius !== null">{{ g.temperature_celsius.toFixed(0) }}°C</template>
          <template v-if="g.utilization_percent !== null"> · {{ g.utilization_percent.toFixed(0) }}% d'utilisation</template>
          <template v-if="g.memory_used_mb !== null && g.memory_total_mb !== null">
            · {{ g.memory_used_mb }} / {{ g.memory_total_mb }} Mo
          </template>
        </span>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.temp-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.temp-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
.temp-card-inner { display: flex; align-items: flex-start; justify-content: space-between; gap: 10px; }
.temp-empty { color: var(--nx-text-secondary); }
.temp-gpu-row { display: flex; justify-content: space-between; gap: 10px; flex-wrap: wrap; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
