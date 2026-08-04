<!-- src/pages/DiskVisualizerPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import { usePreferencesStore } from "@/stores/preferencesStore";

interface UsageEntry { mountpoint: string; total_bytes: number; used_bytes: number; used_percent: number }
interface LargeFile { path: string; size_bytes: number }

const MIN_SIZE_BYTES = 100 * 1024 * 1024; // 100 MB

const preferences = usePreferencesStore();
const usage = ref<UsageEntry[]>([]);
const usageError = ref<string | null>(null);

onMounted(async () => {
  try {
    usage.value = await invoke<UsageEntry[]>("list_disk_usage");
  } catch (e) {
    usageError.value = String(e);
  }
});

function formatGb(bytes: number): string {
  return `${(bytes / 1_073_741_824).toFixed(1)} Go`;
}

const scanDir = ref(preferences.defaultScanDirectory);
const scanning = ref(false);
const scanDone = ref(false);
const scanError = ref<string | null>(null);
const largeFiles = ref<LargeFile[]>([]);

async function scan() {
  scanning.value = true;
  scanDone.value = false;
  scanError.value = null;
  try {
    const results = await invoke<LargeFile[]>("find_large_files_cmd", { directory: scanDir.value, minSizeBytes: MIN_SIZE_BYTES });
    largeFiles.value = [...results].sort((a, b) => b.size_bytes - a.size_bytes);
    scanDone.value = true;
  } catch (e) {
    scanError.value = String(e);
  } finally {
    scanning.value = false;
  }
}

function maxSize(): number {
  return largeFiles.value.length > 0 ? Math.max(...largeFiles.value.map((f) => f.size_bytes)) : 1;
}
</script>

<template>
  <div class="dv-page">
    <NxSectionHeader title="Visualiseur de disque" description="Utilisation par point de montage et plus gros fichiers d'un dossier." />

    <NxCard v-if="usageError" danger>{{ usageError }}</NxCard>

    <NxCard v-for="u in usage" :key="u.mountpoint" class="dv-usage-row">
      <div class="dv-usage-info">
        <span>{{ u.mountpoint }}</span>
        <span>{{ u.used_percent }}% ({{ formatGb(u.used_bytes) }} / {{ formatGb(u.total_bytes) }})</span>
      </div>
      <div class="dv-bar">
        <div class="dv-bar-fill" :style="{ width: `${u.used_percent}%` }"></div>
      </div>
    </NxCard>

    <NxCard class="dv-scan">
      <div class="dv-scan-row">
        <NxInput v-model="scanDir" placeholder="Dossier à analyser (ex: /home/dev)" />
        <NxButton :disabled="scanning || scanDir === ''" @click="scan">{{ scanning ? "Analyse..." : "Analyser" }}</NxButton>
      </div>
      <NxCard v-if="scanError" danger>{{ scanError }}</NxCard>
      <div v-else-if="scanDone && largeFiles.length === 0" class="dv-empty">Aucun gros fichier trouvé.</div>
      <div v-for="f in largeFiles" :key="f.path" class="dv-file-row">
        <span class="dv-file-path">{{ f.path }}</span>
        <div class="dv-file-bar-wrap">
          <div class="dv-file-bar" :style="{ width: `${(f.size_bytes / maxSize()) * 100}%` }"></div>
        </div>
        <span class="dv-file-size">{{ formatGb(f.size_bytes) }}</span>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.dv-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.dv-usage-row { display: flex; flex-direction: column; gap: 6px; }
.dv-usage-info { display: flex; justify-content: space-between; font-size: 13px; }
.dv-bar { height: 8px; border-radius: 4px; background: color-mix(in srgb, var(--nx-accent-primary) 15%, transparent); overflow: hidden; }
.dv-bar-fill { height: 100%; background: var(--nx-accent-primary); border-radius: 4px; }
.dv-scan { display: flex; flex-direction: column; gap: 10px; }
.dv-empty { color: var(--nx-text-secondary); font-size: 13px; }
.dv-scan-row { display: flex; gap: 10px; align-items: center; }
.dv-file-row { display: flex; align-items: center; gap: 10px; padding: 4px 0; font-size: 12px; }
.dv-file-path { min-width: 220px; word-break: break-all; }
.dv-file-bar-wrap { flex: 1; height: 6px; border-radius: 3px; background: color-mix(in srgb, var(--nx-accent-primary) 15%, transparent); overflow: hidden; }
.dv-file-bar { height: 100%; background: var(--nx-accent-primary); border-radius: 3px; }
.dv-file-size { min-width: 60px; text-align: right; color: var(--nx-text-secondary); }
</style>
