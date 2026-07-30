<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface PciDevice { slot: string; class: string; description: string }

const devices = ref<PciDevice[]>([]);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    devices.value = await invoke<PciDevice[]>("get_pci_devices");
    error.value = null;
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  }
});
</script>

<template>
  <div class="hw-page">
    <h1>Composants matériels</h1>
    <div class="hw-error" v-if="error">
      Impossible de récupérer les composants matériels : {{ error }}
    </div>
    <table class="hw-table">
      <thead>
        <tr><th>Slot</th><th>Classe</th><th>Description</th></tr>
      </thead>
      <tbody>
        <tr v-for="d in devices" :key="d.slot">
          <td>{{ d.slot }}</td>
          <td>{{ d.class }}</td>
          <td>{{ d.description }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.hw-page { padding: 24px; color: var(--nx-text-primary); }
.hw-error { margin-top: 16px; padding: 12px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-secondary); }
.hw-table { width: 100%; border-collapse: collapse; margin-top: 16px; font-size: 13px; }
.hw-table th { text-align: left; color: var(--nx-text-secondary); border-bottom: 1px solid var(--nx-border); padding: 8px; }
.hw-table td { padding: 8px; border-bottom: 1px solid var(--nx-border); }
</style>
