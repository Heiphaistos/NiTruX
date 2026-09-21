<!-- src/pages/BackupPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

const sourceDir = ref("");
const creating = ref(false);
const error = ref<string | null>(null);
const resultPath = ref<string | null>(null);

interface BackupEntry { path: string; size_bytes: number; created_epoch_secs: number }

const backups = ref<BackupEntry[] | null>(null);
const listError = ref<string | null>(null);

function formatSize(bytes: number): string {
  const mb = bytes / (1024 * 1024);
  return mb >= 1024 ? `${(mb / 1024).toFixed(1)} Go` : `${mb.toFixed(1)} Mo`;
}

function formatDate(epochSecs: number): string {
  return new Date(epochSecs * 1000).toLocaleString();
}

async function loadBackups() {
  listError.value = null;
  try {
    backups.value = await invoke<BackupEntry[]>("list_backups");
  } catch (e) {
    listError.value = String(e);
  }
}

async function createBackup() {
  creating.value = true;
  error.value = null;
  resultPath.value = null;
  try {
    resultPath.value = await invoke<string>("create_backup", { sourceDir: sourceDir.value });
    await loadBackups();
  } catch (e) {
    error.value = String(e);
  } finally {
    creating.value = false;
  }
}

onMounted(loadBackups);
</script>

<template>
  <div class="bkp-page">
    <NxSectionHeader title="Sauvegarde" description="Archive un dossier vers un fichier .tar.gz horodaté dans votre dossier personnel." />

    <NxCard>
      <div class="bkp-form-row">
        <NxInput v-model="sourceDir" placeholder="Dossier à sauvegarder (ex: /home/dev/documents)" aria-label="Dossier à sauvegarder" />
        <NxButton :disabled="creating || sourceDir === ''" @click="createBackup">{{ creating ? "Sauvegarde en cours..." : "Créer la sauvegarde" }}</NxButton>
      </div>
      <NxCard v-if="error" danger>{{ error }}</NxCard>
      <NxBadge v-if="resultPath" status="success" live>Sauvegarde créée : {{ resultPath }}</NxBadge>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Sauvegardes existantes" :description="backups ? String(backups.length) : ''" />
      <NxCard v-if="listError" danger>{{ listError }}</NxCard>
      <div v-else-if="backups && backups.length === 0" class="bkp-empty">Aucune sauvegarde créée par NiTruX pour l'instant.</div>
      <div v-for="b in backups ?? []" :key="b.path" class="bkp-row">
        <span class="bkp-path">{{ b.path }}</span>
        <span class="bkp-meta">{{ formatDate(b.created_epoch_secs) }} — {{ formatSize(b.size_bytes) }}</span>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.bkp-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.bkp-form-row { display: flex; gap: 10px; align-items: center; }
.bkp-empty { color: var(--nx-text-secondary); font-size: 13px; }
.bkp-row { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 8px; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
.bkp-path { font-family: monospace; word-break: break-all; }
.bkp-meta { color: var(--nx-text-secondary); }
</style>
