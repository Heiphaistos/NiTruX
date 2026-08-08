<!-- src/pages/PortableAppsPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import { portableAppsCatalog, type PortableAppEntry } from "@/data/portableAppsCatalog";

interface PortableAppFile {
  filename: string;
  size_bytes: number;
}

type DownloadState = "idle" | "downloading" | "error";

const downloadState = ref<Record<string, DownloadState>>({});
const downloadErrors = ref<Record<string, string>>({});
const downloaded = ref<PortableAppFile[]>([]);
const loadError = ref<string | null>(null);
const launchError = ref<string | null>(null);

function stateOf(entry: PortableAppEntry): DownloadState {
  return downloadState.value[entry.id] ?? "idle";
}

function bytesToMb(bytes: number): string {
  return (bytes / 1024 / 1024).toFixed(1);
}

async function refreshDownloaded() {
  try {
    downloaded.value = await invoke<PortableAppFile[]>("list_portable_apps");
    loadError.value = null;
  } catch (e) {
    loadError.value = String(e);
  }
}

onMounted(refreshDownloaded);

async function download(entry: PortableAppEntry) {
  downloadState.value[entry.id] = "downloading";
  delete downloadErrors.value[entry.id];
  try {
    await invoke<PortableAppFile>("download_portable_app", {
      owner: entry.githubOwner,
      repo: entry.githubRepo,
    });
    downloadState.value[entry.id] = "idle";
    await refreshDownloaded();
  } catch (e) {
    downloadState.value[entry.id] = "error";
    downloadErrors.value[entry.id] = String(e);
  }
}

async function launch(file: PortableAppFile) {
  launchError.value = null;
  try {
    await invoke("launch_portable_app", { filename: file.filename });
  } catch (e) {
    launchError.value = String(e);
  }
}

async function remove(file: PortableAppFile) {
  launchError.value = null;
  try {
    await invoke("remove_portable_app", { filename: file.filename });
    await refreshDownloaded();
  } catch (e) {
    launchError.value = String(e);
  }
}
</script>

<template>
  <div class="pa-page">
    <NxSectionHeader
      title="Apps portables"
      description="AppImage : exécutables autonomes qui ne nécessitent aucune installation, ni de privilèges root."
    />

    <NxCard v-if="downloaded.length > 0">
      <NxSectionHeader title="Applications téléchargées" />
      <NxCard v-if="loadError" danger>{{ loadError }}</NxCard>
      <NxCard v-if="launchError" danger>{{ launchError }}</NxCard>
      <div v-for="file in downloaded" :key="file.filename" class="pa-file-row">
        <span class="pa-file-name">{{ file.filename }}</span>
        <span class="pa-file-size">{{ bytesToMb(file.size_bytes) }} Mo</span>
        <div class="pa-file-actions">
          <NxButton @click="launch(file)">Lancer</NxButton>
          <NxButton variant="danger" @click="remove(file)">Supprimer</NxButton>
        </div>
      </div>
    </NxCard>

    <div class="pa-grid">
      <NxCard v-for="entry in portableAppsCatalog" :key="entry.id" class="pa-card">
        <div class="pa-card-header">
          <span class="pa-icon">{{ entry.icon }}</span>
          <div>
            <div class="pa-name">{{ entry.name }}</div>
            <div class="pa-desc">{{ entry.description }}</div>
          </div>
        </div>
        <NxCard v-if="stateOf(entry) === 'error'" danger class="pa-error">{{ downloadErrors[entry.id] }}</NxCard>
        <NxButton :disabled="stateOf(entry) === 'downloading'" @click="download(entry)">
          {{ stateOf(entry) === "downloading" ? "Téléchargement..." : "Télécharger" }}
        </NxButton>
      </NxCard>
    </div>
  </div>
</template>

<style scoped>
.pa-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.pa-file-row { display: flex; align-items: center; gap: 12px; padding: 6px 0; font-size: 13px; flex-wrap: wrap; }
.pa-file-name { font-weight: 600; flex: 1; min-width: 160px; }
.pa-file-size { color: var(--nx-text-secondary); }
.pa-file-actions { display: flex; gap: 8px; }
.pa-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 14px; }
.pa-card { display: flex; flex-direction: column; gap: 10px; }
.pa-card-header { display: flex; gap: 10px; align-items: flex-start; }
.pa-icon { font-size: 28px; line-height: 1; }
.pa-name { font-weight: 600; color: var(--nx-text-primary); }
.pa-desc { font-size: 12px; color: var(--nx-text-secondary); }
.pa-error { font-size: 12px; padding: 8px 10px; }
</style>
