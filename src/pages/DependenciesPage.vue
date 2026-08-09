<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import NxBadge from "@/components/ui/NxBadge.vue";

interface MissingDependency { binary: string; missing_library: string }

const results = ref<MissingDependency[] | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    results.value = await invoke<MissingDependency[]>("scan_missing_dependencies");
  } catch (e) {
    error.value = String(e);
  }
});
</script>

<template>
  <div class="dep-page">
    <NxSectionHeader title="Dépendances" description="Vérifie qu'un ensemble de binaires système courants ont toutes leurs bibliothèques partagées résolues." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <div v-if="results && results.length === 0" class="dep-empty">Aucune dépendance manquante détectée.</div>

    <NxCard v-else-if="results">
      <div v-for="(r, i) in results" :key="`${r.binary}-${r.missing_library}-${i}`" class="dep-row">
        <span>{{ r.binary }}</span>
        <NxBadge status="danger">{{ r.missing_library }}</NxBadge>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.dep-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.dep-empty { color: var(--nx-text-secondary); }
.dep-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
