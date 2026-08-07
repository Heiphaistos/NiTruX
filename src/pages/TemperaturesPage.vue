<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxStatTile from "@/components/ui/NxStatTile.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface TemperatureReading { label: string; celsius: number }
interface SensorSnapshot { battery_percent: number | null; battery_charging: boolean | null; temperatures: TemperatureReading[] }

const snapshot = ref<SensorSnapshot | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    snapshot.value = await invoke<SensorSnapshot>("get_sensor_snapshot");
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
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
  </div>
</template>

<style scoped>
.temp-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.temp-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
.temp-card-inner { display: flex; align-items: flex-start; justify-content: space-between; gap: 10px; }
.temp-empty { color: var(--nx-text-secondary); }
</style>
