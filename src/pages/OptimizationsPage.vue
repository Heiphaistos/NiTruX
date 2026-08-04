<!-- src/pages/OptimizationsPage.vue -->
<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxStatTile from "@/components/ui/NxStatTile.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface OptimizationSnapshot {
  enabled_services: string[];
  swappiness: number | null;
  zram_active: boolean;
  fstrim_timer_enabled: boolean;
}

const snapshot = ref<OptimizationSnapshot | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    snapshot.value = await invoke<OptimizationSnapshot>("get_optimization_snapshot");
  } catch (e) {
    error.value = String(e);
  }
});

const swappinessAdvice = computed(() => {
  const s = snapshot.value?.swappiness;
  if (s === null || s === undefined) return null;
  if (s > 30) return `Swappiness à ${s} — valeur élevée pour un poste de travail. Une valeur plus basse (10-20) privilégie la RAM au swap sur une machine avec suffisamment de mémoire.`;
  return `Swappiness à ${s} — valeur déjà basse, cohérente avec un usage desktop.`;
});
</script>

<template>
  <div class="opt-page">
    <NxSectionHeader title="Optimisations" description="Diagnostic système en lecture seule — aucune action n'est appliquée automatiquement." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <template v-if="snapshot">
      <div class="opt-stats">
        <NxCard><NxStatTile label="Swappiness" :value="String(snapshot.swappiness ?? '—')" /></NxCard>
        <NxCard>
          <NxStatTile label="ZRAM" :value="snapshot.zram_active ? 'actif' : 'inactif'" />
          <NxBadge :status="snapshot.zram_active ? 'success' : 'info'">{{ snapshot.zram_active ? "actif" : "inactif" }}</NxBadge>
        </NxCard>
        <NxCard>
          <NxStatTile label="Timer fstrim" :value="snapshot.fstrim_timer_enabled ? 'activé' : 'désactivé'" />
          <NxBadge :status="snapshot.fstrim_timer_enabled ? 'success' : 'warning'">{{ snapshot.fstrim_timer_enabled ? "activé" : "désactivé" }}</NxBadge>
        </NxCard>
      </div>

      <NxCard v-if="swappinessAdvice">
        <p class="opt-advice">{{ swappinessAdvice }}</p>
      </NxCard>

      <NxCard>
        <NxSectionHeader :title="`Services activés au démarrage (${snapshot.enabled_services.length})`" />
        <div class="opt-services">
          <NxBadge v-for="s in snapshot.enabled_services" :key="s" status="info">{{ s }}</NxBadge>
        </div>
        <div v-if="snapshot.enabled_services.length === 0" class="opt-empty">Aucun service activé au démarrage.</div>
      </NxCard>
    </template>
  </div>
</template>

<style scoped>
.opt-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.opt-stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; }
.opt-advice { margin: 0; font-size: 13px; color: var(--nx-text-secondary); line-height: 1.5; }
.opt-services { display: flex; flex-wrap: wrap; gap: 6px; }
.opt-empty { color: var(--nx-text-secondary); font-size: 13px; }
</style>
