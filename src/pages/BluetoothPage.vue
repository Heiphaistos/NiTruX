<!-- src/pages/BluetoothPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface BluetoothDevice { address: string; name: string }
interface BluetoothStatus { adapter_present: boolean; powered: boolean; devices: BluetoothDevice[]; tool_error: string | null }

const status = ref<BluetoothStatus | null>(null);
const error = ref<string | null>(null);
const toggling = ref(false);
const toggleError = ref<string | null>(null);

async function load() {
  try {
    status.value = await invoke<BluetoothStatus>("get_bluetooth_status");
  } catch (e) {
    error.value = String(e);
  }
}

async function togglePower() {
  if (!status.value) return;
  toggling.value = true;
  toggleError.value = null;
  try {
    await invoke<string>("set_bluetooth_power", { on: !status.value.powered });
    await load();
  } catch (e) {
    toggleError.value = String(e);
  } finally {
    toggling.value = false;
  }
}

onMounted(load);
</script>

<template>
  <div class="bt-page">
    <NxSectionHeader title="Bluetooth" description="Statut de l'adaptateur et périphériques appairés (lecture seule)." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <!-- `bluetoothctl` absent is not the same thing as "no adapter": the
         first is fixable by installing a package, the second is hardware. -->
    <NxCard v-if="status && status.tool_error" danger>{{ status.tool_error }}</NxCard>

    <div v-else-if="status && !status.adapter_present" class="bt-empty">Aucun adaptateur Bluetooth détecté.</div>

    <template v-else-if="status">
      <NxCard class="bt-power">
        <NxBadge :status="status.powered ? 'success' : 'warning'">{{ status.powered ? "activé" : "désactivé" }}</NxBadge>
        <NxButton :disabled="toggling" @click="togglePower">
          {{ toggling ? "…" : status.powered ? "Désactiver" : "Activer" }}
        </NxButton>
      </NxCard>

      <NxCard v-if="toggleError" danger>{{ toggleError }}</NxCard>

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
.bt-power { display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap; }
.bt-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; }
.bt-address { color: var(--nx-text-secondary); font-size: 12px; }
</style>
