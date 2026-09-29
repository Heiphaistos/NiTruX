<!-- src/pages/SensorsPage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import { usePreferencesStore } from "@/stores/preferencesStore";

interface Reading { kind: string; label: string; value: number; unit: string; max: number | null; critical: number | null; alarm: boolean }
interface Chip { chip: string; category: string; device: string; readings: Reading[] }

const preferences = usePreferencesStore();
const chips = ref<Chip[]>([]);
const error = ref<string | null>(null);
let intervalId: number | undefined;
let unmounted = false;

const CATEGORY_LABELS: Record<string, string> = {
  cpu: "Processeur",
  gpu: "Carte graphique",
  storage: "Stockage",
  memory: "Mémoire",
  motherboard: "Carte mère",
  cooling: "Refroidissement / Watercooling",
  psu: "Alimentation",
  battery: "Batterie",
  network: "Réseau",
  other: "Autres",
};

async function refresh() {
  try {
    chips.value = await invoke<Chip[]>("get_all_sensors");
    error.value = null;
  } catch (e) {
    error.value = String(e);
  }
}

onMounted(async () => {
  await refresh();
  if (unmounted) return;
  intervalId = window.setInterval(refresh, Math.max(2000, preferences.dashboardRefreshIntervalMs));
});
onUnmounted(() => {
  unmounted = true;
  if (intervalId) window.clearInterval(intervalId);
});

const totalReadings = computed(() => chips.value.reduce((n, c) => n + c.readings.length, 0));

function tempStatus(r: Reading): "success" | "warning" | "danger" {
  if (r.kind !== "temperature") return "success";
  const crit = r.critical ?? 90;
  if (r.alarm || r.value >= crit) return "danger";
  if (r.value >= crit - 15) return "warning";
  return "success";
}

function formatValue(r: Reading): string {
  const v = r.unit === "RPM" || r.unit === "%" ? Math.round(r.value) : r.value;
  return `${v} ${r.unit}`;
}
</script>

<template>
  <div class="se-page">
    <NxSectionHeader
      title="Capteurs"
      description="Toutes les températures, ventilateurs, pompes, tensions et puissances remontés par le noyau — CPU, GPU, SSD, carte mère, watercooling, alimentation."
    />
    <NxCard v-if="error" danger>{{ error }}</NxCard>
    <p v-else-if="chips.length === 0" class="se-muted">Aucun capteur détecté sur ce système.</p>
    <p v-else class="se-muted">{{ totalReadings }} mesure(s) sur {{ chips.length }} puce(s), mise à jour en direct.</p>

    <NxCard v-for="(chip, i) in chips" :key="i" class="se-chip">
      <div class="se-chip-head">
        <NxBadge status="info">{{ CATEGORY_LABELS[chip.category] ?? chip.category }}</NxBadge>
        <strong>{{ chip.device || chip.chip }}</strong>
        <span class="se-muted se-chipname">{{ chip.chip }}</span>
      </div>
      <div class="se-readings">
        <div v-for="(r, j) in chip.readings" :key="j" class="se-reading" :class="`se-${tempStatus(r)}`">
          <span class="se-label">{{ r.label }}</span>
          <span class="se-value">
            {{ formatValue(r) }}
            <NxBadge v-if="r.alarm" status="danger">alarme</NxBadge>
          </span>
          <span v-if="r.critical" class="se-muted">crit. {{ r.critical }} {{ r.unit }}</span>
        </div>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.se-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.se-muted { font-size: 12px; color: var(--nx-text-secondary); margin: 0; }
.se-chip { display: flex; flex-direction: column; gap: 10px; }
.se-chip-head { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
.se-chipname { margin-left: auto; }
.se-readings { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 8px; }
.se-reading { display: flex; flex-direction: column; gap: 2px; padding: 8px 10px; border-radius: var(--nx-style-radius); background: var(--nx-bg-elevated); border-left: 3px solid var(--nx-accent-success); }
.se-reading.se-warning { border-left-color: var(--nx-accent-warning); }
.se-reading.se-danger { border-left-color: var(--nx-accent-danger); }
.se-label { font-size: 12px; color: var(--nx-text-secondary); }
.se-value { font-size: 18px; font-weight: 700; display: flex; align-items: center; gap: 6px; }
</style>
