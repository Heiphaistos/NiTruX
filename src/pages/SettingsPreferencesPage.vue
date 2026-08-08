<script setup lang="ts">
import { usePreferencesStore } from "@/stores/preferencesStore";
import NxCard from "@/components/ui/NxCard.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSelect from "@/components/ui/NxSelect.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

const preferences = usePreferencesStore();

const REFRESH_INTERVAL_OPTIONS = [
  { value: "1000", label: "1 seconde" },
  { value: "2000", label: "2 secondes" },
  { value: "5000", label: "5 secondes" },
];

function onIntervalChange(value: string) {
  preferences.setDashboardRefreshIntervalMs(Number(value));
}

function onScanDirChange(value: string) {
  preferences.setDefaultScanDirectory(value);
}

// Number("") is 0, not NaN -- Number.isFinite alone let a cleared field
// through as a real "0" write, which the store's clampThreshold() then
// clamped to 1, silently stomping the last valid threshold every time the
// user selected-all-and-retyped. The raw string must be checked for empty
// first; the store's clampThreshold() remains the guard for genuinely
// out-of-range typed values (0, 150, etc).
function readThreshold(event: Event): number | null {
  const raw = (event.target as HTMLInputElement).value;
  if (raw.trim() === "") return null;
  const value = Number(raw);
  return Number.isFinite(value) ? value : null;
}
function onCpuThresholdChange(event: Event) {
  const value = readThreshold(event);
  if (value !== null) preferences.setCpuAlertThreshold(value);
}
function onRamThresholdChange(event: Event) {
  const value = readThreshold(event);
  if (value !== null) preferences.setRamAlertThreshold(value);
}
function onDiskThresholdChange(event: Event) {
  const value = readThreshold(event);
  if (value !== null) preferences.setDiskAlertThreshold(value);
}
</script>

<template>
  <div class="pref-page">
    <NxSectionHeader title="Préférences" description="Réglages de l'application (pas de la configuration système)." />

    <NxCard class="pref-card">
      <label class="pref-label">Répertoire par défaut pour les scanners</label>
      <NxInput
        :model-value="preferences.defaultScanDirectory"
        placeholder="ex: /home/dev"
        aria-label="Répertoire par défaut pour les scanners"
        @update:model-value="onScanDirChange"
      />
    </NxCard>

    <NxCard class="pref-card">
      <label class="pref-label">Intervalle de rafraîchissement du tableau de bord</label>
      <NxSelect
        :model-value="String(preferences.dashboardRefreshIntervalMs)"
        :options="REFRESH_INTERVAL_OPTIONS"
        aria-label="Intervalle de rafraîchissement du tableau de bord"
        @update:model-value="onIntervalChange"
      />
    </NxCard>

    <NxCard class="pref-card">
      <label class="pref-label">Seuils d'alerte du tableau de bord (%)</label>
      <div class="pref-threshold-row">
        <label class="pref-threshold-field">
          <span>CPU</span>
          <input
            type="number"
            min="1"
            max="100"
            :value="preferences.cpuAlertThreshold"
            aria-label="Seuil d'alerte CPU"
            class="pref-threshold-input"
            @change="onCpuThresholdChange"
          />
        </label>
        <label class="pref-threshold-field">
          <span>RAM</span>
          <input
            type="number"
            min="1"
            max="100"
            :value="preferences.ramAlertThreshold"
            aria-label="Seuil d'alerte RAM"
            class="pref-threshold-input"
            @change="onRamThresholdChange"
          />
        </label>
        <label class="pref-threshold-field">
          <span>Disque</span>
          <input
            type="number"
            min="1"
            max="100"
            :value="preferences.diskAlertThreshold"
            aria-label="Seuil d'alerte disque"
            class="pref-threshold-input"
            @change="onDiskThresholdChange"
          />
        </label>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.pref-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.pref-card { display: flex; flex-direction: column; gap: 8px; }
.pref-label { font-size: 13px; color: var(--nx-text-secondary); display: flex; align-items: center; gap: 8px; }
.pref-threshold-row { display: flex; gap: 16px; flex-wrap: wrap; }
.pref-threshold-field { display: flex; align-items: center; gap: 8px; font-size: 13px; color: var(--nx-text-secondary); }
.pref-threshold-input {
  width: 64px;
  padding: 6px 8px;
  border-radius: var(--nx-style-radius);
  border: var(--nx-style-border-width) solid var(--nx-style-border-color);
  background: var(--nx-style-bg);
  color: var(--nx-text-primary);
  font-family: var(--nx-style-font-family);
  font-size: 13px;
}
</style>
