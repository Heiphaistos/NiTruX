<!-- src/pages/PeripheralsPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface AudioSink { name: string; driver: string; state: string }
interface PrinterInfo { name: string; status: string }

const monitors = ref<string[] | null>(null);
const usbDevices = ref<string[] | null>(null);
const audioSinks = ref<AudioSink[] | null>(null);
const printers = ref<PrinterInfo[] | null>(null);

onMounted(async () => {
  monitors.value = await invoke<string[]>("get_monitors");
  usbDevices.value = await invoke<string[]>("get_usb_devices");
  audioSinks.value = await invoke<AudioSink[]>("get_audio_sinks");
  printers.value = await invoke<PrinterInfo[]>("get_printers");
});
</script>

<template>
  <div class="ph-page">
    <NxSectionHeader title="Périphériques" description="Moniteurs, USB, audio et imprimantes." />

    <NxCard>
      <NxSectionHeader title="Moniteurs" />
      <div v-if="monitors && monitors.length === 0" class="ph-empty">Aucun moniteur détecté.</div>
      <div v-for="m in monitors ?? []" :key="m" class="ph-row">{{ m }}</div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="USB" />
      <div v-if="usbDevices && usbDevices.length === 0" class="ph-empty">Aucun périphérique USB détecté.</div>
      <div v-for="u in usbDevices ?? []" :key="u" class="ph-row">{{ u }}</div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Audio" />
      <div v-if="audioSinks && audioSinks.length === 0" class="ph-empty">Aucune sortie audio détectée.</div>
      <div v-for="a in audioSinks ?? []" :key="a.name" class="ph-row">{{ a.name }} ({{ a.driver }}) — {{ a.state }}</div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Imprimantes" />
      <div v-if="printers && printers.length === 0" class="ph-empty">Aucune imprimante détectée.</div>
      <div v-for="p in printers ?? []" :key="p.name" class="ph-row">{{ p.name }} — {{ p.status }}</div>
    </NxCard>
  </div>
</template>

<style scoped>
.ph-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.ph-empty { color: var(--nx-text-secondary); font-size: 13px; }
.ph-row { padding: 4px 0; font-size: 13px; }
</style>
