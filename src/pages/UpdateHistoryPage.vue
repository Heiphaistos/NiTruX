<!-- src/pages/UpdateHistoryPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface UpdateHistoryEntry { start_date: string; commandline: string; summary: string }

const entries = ref<UpdateHistoryEntry[] | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    entries.value = await invoke<UpdateHistoryEntry[]>("get_update_history");
  } catch (e) {
    error.value = String(e);
  }
});
</script>

<template>
  <div class="uh-page">
    <NxSectionHeader title="Historique des mises à jour" description="Journal des installations et mises à jour passées." />
    <NxCard v-if="error" danger>{{ error }}</NxCard>
    <NxCard v-for="e in entries ?? []" :key="e.start_date" class="uh-row">
      <div class="uh-date">{{ e.start_date }}</div>
      <div class="uh-cmd">{{ e.commandline }}</div>
      <div class="uh-summary">{{ e.summary }}</div>
    </NxCard>
  </div>
</template>

<style scoped>
.uh-page { padding: 24px; display: flex; flex-direction: column; gap: 10px; }
.uh-row { font-size: 13px; display: flex; flex-direction: column; gap: 4px; }
.uh-date { color: var(--nx-text-secondary); font-size: 11px; }
.uh-cmd { font-weight: 600; }
.uh-summary { color: var(--nx-text-secondary); font-size: 12px; }
</style>
