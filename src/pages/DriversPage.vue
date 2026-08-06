<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxStatTile from "@/components/ui/NxStatTile.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import NxBadge from "@/components/ui/NxBadge.vue";

interface DeviceDriver { slot: string; description: string; driver: string | null }
interface DriverSnapshot { loaded_modules: string[]; gpu_driver: string; devices: DeviceDriver[] }

const snapshot = ref<DriverSnapshot | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    snapshot.value = await invoke<DriverSnapshot>("get_driver_snapshot");
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  }
});
</script>

<template>
  <div class="drv-page">
    <NxSectionHeader title="Pilotes" description="Pilotes noyau actifs par périphérique PCI et modules chargés." />

    <NxCard v-if="error" danger>Impossible de récupérer les pilotes : {{ error }}</NxCard>

    <template v-if="snapshot">
      <div class="drv-stats">
        <NxCard><NxStatTile label="Pilote GPU actif" :value="snapshot.gpu_driver" /></NxCard>
        <NxCard><NxStatTile label="Modules chargés" :value="String(snapshot.loaded_modules.length)" /></NxCard>
        <NxCard><NxStatTile label="Périphériques PCI" :value="String(snapshot.devices.length)" /></NxCard>
      </div>

      <NxCard class="drv-note">
        <p>
          Sur Linux, il n'existe pas de mécanisme séparé de « mise à jour de pilote » comme sur Windows : les pilotes sont
          soit intégrés au noyau, soit installés comme des modules via le <strong>gestionnaire de paquets</strong> du système
          (voir <strong>Maintenance &gt; Mises à jour</strong>). Un pilote à jour est simplement un système à jour.
        </p>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Périphériques & pilotes" />
        <div class="drv-table-scroll">
          <table class="drv-table">
            <thead>
              <tr><th>Emplacement</th><th>Périphérique</th><th>Pilote</th></tr>
            </thead>
            <tbody>
              <tr v-for="d in snapshot.devices" :key="d.slot">
                <td>{{ d.slot }}</td>
                <td>{{ d.description }}</td>
                <td>
                  <NxBadge v-if="d.driver" status="success">{{ d.driver }}</NxBadge>
                  <NxBadge v-else status="warning">aucun</NxBadge>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Modules chargés" :description="String(snapshot.loaded_modules.length)" />
        <div class="drv-modules">
          <NxBadge v-for="mod in snapshot.loaded_modules" :key="mod" status="info">{{ mod }}</NxBadge>
        </div>
      </NxCard>
    </template>
  </div>
</template>

<style scoped>
.drv-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.drv-stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; }
.drv-note p { margin: 0; font-size: 13px; color: var(--nx-text-secondary); line-height: 1.5; }
.drv-table-scroll { overflow-x: auto; }
.drv-table { width: 100%; min-width: 480px; border-collapse: collapse; font-size: 13px; }
.drv-table th { text-align: left; color: var(--nx-text-secondary); border-bottom: 1px solid var(--nx-style-border-color); padding: 8px; }
.drv-table td { padding: 8px; border-bottom: 1px solid var(--nx-style-border-color); }
.drv-modules { display: flex; flex-wrap: wrap; gap: 6px; }
</style>
