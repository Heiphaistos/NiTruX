<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface DriverSnapshot { loaded_modules: string[]; gpu_driver: string }

const snapshot = ref<DriverSnapshot | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    snapshot.value = await invoke<DriverSnapshot>("get_driver_snapshot");
    error.value = null;
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  }
});
</script>

<template>
  <div class="drv-page">
    <h1>Pilotes & modules noyau</h1>
    <div class="drv-error" v-if="error">
      Impossible de récupérer les pilotes et modules noyau : {{ error }}
    </div>
    <template v-if="snapshot">
      <div class="drv-gpu">Pilote GPU actif : <strong>{{ snapshot.gpu_driver }}</strong></div>
      <h2>Modules chargés ({{ snapshot.loaded_modules.length }})</h2>
      <ul class="drv-list">
        <li v-for="mod in snapshot.loaded_modules" :key="mod">{{ mod }}</li>
      </ul>
    </template>
  </div>
</template>

<style scoped>
.drv-page { padding: 24px; color: var(--nx-text-primary); }
.drv-error { margin-top: 16px; padding: 12px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-secondary); }
.drv-gpu { margin: 12px 0; padding: 10px 14px; border-radius: 8px; background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); }
.drv-list { columns: 3; column-gap: 24px; font-size: 13px; margin-top: 8px; }
.drv-list li { padding: 3px 0; }
</style>
