<!-- src/pages/BluetoothPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface BluetoothDevice { address: string; name: string }
interface BluetoothStatus { adapter_present: boolean; powered: boolean; devices: BluetoothDevice[] }

const status = ref<BluetoothStatus | null>(null);

onMounted(async () => {
  status.value = await invoke<BluetoothStatus>("get_bluetooth_status");
});
</script>

<template>
  <div class="bt-page">
    <NxSectionHeader title="Bluetooth" description="Statut de l'adaptateur et périphériques appairés (lecture seule)." />

    <div v-if="status && !status.adapter_present" class="bt-empty">Aucun adaptateur Bluetooth détecté.</div>

    <template v-else-if="status">
      <NxCard>
        <NxBadge :status="status.powered ? 'success' : 'warning'">{{ status.powered ? "activé" : "désactivé" }}</NxBadge>
      </NxCard>

      <NxCard v-if="status.devices.length === 0" class="bt-empty">Aucun périphérique Bluetooth appairé.</NxCard>

      <NxCard v-for="d in status.devices" :key="d.address" class="bt-row">
        <span>{{ d.name }}</span>
        <span class="bt-address">{{ d.address }}</span>
      </NxCard>
    </template>
  </div>
</template>

<style scoped>
.bt-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.bt-empty { color: var(--nx-text-secondary); }
.bt-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; }
.bt-address { color: var(--nx-text-secondary); font-size: 12px; }
</style>
