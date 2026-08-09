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

const monitorsError = ref<string | null>(null);
const usbError = ref<string | null>(null);
const audioError = ref<string | null>(null);
const printersError = ref<string | null>(null);

// 4 independent try/catch blocks, not one shared try: these are 4
// unrelated hardware categories (xrandr/lsusb/pactl/lpstat) -- a shared
// try would mean one section failing (e.g. no CUPS, so lpstat errors)
// leaves the other three sections' data unset too, even though each
// command is otherwise independent and would have succeeded on its own.
// Mirrors the already-established pattern in InstalledSoftwarePage.vue.
onMounted(async () => {
  try {
    monitors.value = await invoke<string[]>("get_monitors");
  } catch (e) {
    monitorsError.value = String(e);
  }
  try {
    usbDevices.value = await invoke<string[]>("get_usb_devices");
  } catch (e) {
    usbError.value = String(e);
  }
  try {
    audioSinks.value = await invoke<AudioSink[]>("get_audio_sinks");
  } catch (e) {
    audioError.value = String(e);
  }
  try {
    printers.value = await invoke<PrinterInfo[]>("get_printers");
  } catch (e) {
    printersError.value = String(e);
  }
});
</script>

<template>
  <div class="ph-page">
    <NxSectionHeader title="Périphériques" description="Moniteurs, USB, audio et imprimantes." />

    <NxCard>
      <NxSectionHeader title="Moniteurs" />
      <NxCard v-if="monitorsError" danger>{{ monitorsError }}</NxCard>
      <div v-if="monitors && monitors.length === 0" class="ph-empty">Aucun moniteur détecté.</div>
      <div v-for="m in monitors ?? []" :key="m" class="ph-row">{{ m }}</div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="USB" />
      <NxCard v-if="usbError" danger>{{ usbError }}</NxCard>
      <div v-if="usbDevices && usbDevices.length === 0" class="ph-empty">Aucun périphérique USB détecté.</div>
      <div v-for="u in usbDevices ?? []" :key="u" class="ph-row">{{ u }}</div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Audio" />
      <NxCard v-if="audioError" danger>{{ audioError }}</NxCard>
      <div v-if="audioSinks && audioSinks.length === 0" class="ph-empty">Aucune sortie audio détectée.</div>
      <div v-for="a in audioSinks ?? []" :key="a.name" class="ph-row">{{ a.name }} ({{ a.driver }}) — {{ a.state }}</div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Imprimantes" />
      <NxCard v-if="printersError" danger>{{ printersError }}</NxCard>
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
