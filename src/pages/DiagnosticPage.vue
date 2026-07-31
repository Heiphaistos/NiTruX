<!-- src/pages/DiagnosticPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

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
  <div class="diag-page">
    <NxSectionHeader title="Diagnostic" description="Composants matériels détectés (PCI)." />
    <NxCard v-if="error" danger>
      Impossible de récupérer les composants matériels : {{ error }}
    </NxCard>
    <NxCard v-if="devices.length">
      <table class="diag-table">
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
    </NxCard>
  </div>
</template>

<style scoped>
.diag-page { padding: 24px; }
.diag-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.diag-table th { text-align: left; color: var(--nx-text-secondary); border-bottom: 1px solid var(--nx-style-border-color); padding: 8px; }
.diag-table td { padding: 8px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
