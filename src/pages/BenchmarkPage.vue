<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxStatTile from "@/components/ui/NxStatTile.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface BenchmarkResult {
  cpu_hashes_per_sec: number;
  disk_write_mbps: number;
  disk_read_mbps: number;
  memory_bandwidth_gbps: number;
}

const running = ref(false);
const error = ref<string | null>(null);
const result = ref<BenchmarkResult | null>(null);

async function run() {
  running.value = true;
  error.value = null;
  result.value = null;
  try {
    result.value = await invoke<BenchmarkResult>("run_benchmark");
  } catch (e) {
    error.value = String(e);
  } finally {
    running.value = false;
  }
}
</script>

<template>
  <div class="bench-page">
    <NxSectionHeader title="Benchmark" description="Mesure rapide des performances CPU, disque et mémoire de ce système." />

    <NxCard>
      <NxButton :disabled="running" @click="run">{{ running ? "Benchmark en cours..." : "Lancer le benchmark" }}</NxButton>
    </NxCard>

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <div class="bench-grid" v-if="result">
      <NxCard><NxStatTile label="CPU (hachages/s)" :value="result.cpu_hashes_per_sec.toLocaleString('fr-FR')" /></NxCard>
      <NxCard><NxStatTile label="Écriture disque" :value="`${result.disk_write_mbps.toFixed(1)} Mo/s`" /></NxCard>
      <NxCard><NxStatTile label="Lecture disque" :value="`${result.disk_read_mbps.toFixed(1)} Mo/s`" /></NxCard>
      <NxCard><NxStatTile label="Bande passante mémoire" :value="`${result.memory_bandwidth_gbps.toFixed(1)} Go/s`" /></NxCard>
    </div>
  </div>
</template>

<style scoped>
.bench-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.bench-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
</style>
