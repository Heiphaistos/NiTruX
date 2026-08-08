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
  </div>
</template>

<style scoped>
.pref-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.pref-card { display: flex; flex-direction: column; gap: 8px; }
.pref-label { font-size: 13px; color: var(--nx-text-secondary); display: flex; align-items: center; gap: 8px; }
</style>
