<!-- src/pages/FileToolsPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSelect from "@/components/ui/NxSelect.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import { usePreferencesStore } from "@/stores/preferencesStore";

interface DuplicateGroup { hash: string; paths: string[]; size_bytes: number }
interface LargeFile { path: string; size_bytes: number }

type Tab = "duplicates" | "largefiles" | "hashcheck";
const activeTab = ref<Tab>("duplicates");

const preferences = usePreferencesStore();
const scanDir = ref(preferences.defaultScanDirectory);
const duplicateGroups = ref<DuplicateGroup[]>([]);
const duplicatesError = ref<string | null>(null);
const duplicatesLoading = ref(false);

async function scanDuplicates() {
  duplicatesLoading.value = true;
  duplicatesError.value = null;
  try {
    duplicateGroups.value = await invoke<DuplicateGroup[]>("find_duplicate_files", { directory: scanDir.value });
  } catch (e) {
    duplicatesError.value = String(e);
  } finally {
    duplicatesLoading.value = false;
  }
}

const largeFileDir = ref(preferences.defaultScanDirectory);
const minSizeMb = ref("100");
const largeFiles = ref<LargeFile[]>([]);
const largeFilesError = ref<string | null>(null);
const largeFilesLoading = ref(false);

async function scanLargeFiles() {
  largeFilesError.value = null;
  // "MB min" is a free-text NxInput (no type="number" guard at the DOM
  // level), so an empty/non-numeric/negative value must be caught here --
  // otherwise Number() silently produces NaN and the invoke call fails with
  // a raw Tauri IPC deserialization error instead of a message the user can
  // act on. The only other Number() conversion in the app (preferences
  // refresh interval) is always fed by a fixed <select>, never free text.
  const minSizeMbValue = Number(minSizeMb.value);
  if (!Number.isFinite(minSizeMbValue) || minSizeMbValue < 0) {
    largeFilesError.value = "Veuillez entrer une taille minimale valide (nombre positif, en Mo).";
    return;
  }
  largeFilesLoading.value = true;
  try {
    largeFiles.value = await invoke<LargeFile[]>("find_large_files_cmd", {
      directory: largeFileDir.value,
      minSizeBytes: minSizeMbValue * 1024 * 1024,
    });
  } catch (e) {
    largeFilesError.value = String(e);
  } finally {
    largeFilesLoading.value = false;
  }
}

const HASH_OPTIONS = [
  { value: "sha256", label: "SHA-256" },
  { value: "sha1", label: "SHA-1" },
  { value: "md5", label: "MD5" },
];

const hashPath = ref("");
const hashAlgorithm = ref<"sha256" | "sha1" | "md5">("sha256");
const hashResult = ref<string | null>(null);
const hashError = ref<string | null>(null);
const expectedHash = ref("");
const verifyMatch = ref<boolean | null>(null);

async function computeHash() {
  hashError.value = null;
  hashResult.value = null;
  try {
    hashResult.value = await invoke<string>("compute_file_hash", { path: hashPath.value, algorithm: hashAlgorithm.value });
  } catch (e) {
    hashError.value = String(e);
  }
}

async function verifyHash() {
  hashError.value = null;
  verifyMatch.value = null;
  try {
    verifyMatch.value = await invoke<boolean>("verify_file_hash", {
      path: hashPath.value,
      algorithm: hashAlgorithm.value,
      expected: expectedHash.value,
    });
  } catch (e) {
    hashError.value = String(e);
  }
}

function bytesToMb(bytes: number): string {
  return (bytes / 1024 / 1024).toFixed(1);
}
</script>

<template>
  <div class="ft-page">
    <NxSectionHeader title="Outils fichiers" description="Doublons, gros fichiers et vérification d'intégrité." />

    <div class="ft-tabs">
      <button :class="{ active: activeTab === 'duplicates' }" @click="activeTab = 'duplicates'">Doublons</button>
      <button :class="{ active: activeTab === 'largefiles' }" @click="activeTab = 'largefiles'">Gros fichiers</button>
      <button :class="{ active: activeTab === 'hashcheck' }" @click="activeTab = 'hashcheck'">Vérif. hash</button>
    </div>

    <NxCard v-if="activeTab === 'duplicates'">
      <div class="ft-form-row">
        <NxInput v-model="scanDir" placeholder="Dossier à scanner..." />
        <NxButton :disabled="duplicatesLoading" @click="scanDuplicates">{{ duplicatesLoading ? "Analyse..." : "Rechercher" }}</NxButton>
      </div>
      <NxCard v-if="duplicatesError" danger>{{ duplicatesError }}</NxCard>
      <div v-for="g in duplicateGroups" :key="g.hash" class="ft-dup-group">
        <div>{{ g.paths.length }} fichiers identiques ({{ bytesToMb(g.size_bytes) }} Mo chacun)</div>
        <ul><li v-for="p in g.paths" :key="p">{{ p }}</li></ul>
      </div>
    </NxCard>

    <NxCard v-else-if="activeTab === 'largefiles'">
      <div class="ft-form-row">
        <NxInput v-model="largeFileDir" placeholder="Dossier à scanner..." />
        <NxInput v-model="minSizeMb" placeholder="Mo min" />
        <NxButton :disabled="largeFilesLoading" @click="scanLargeFiles">{{ largeFilesLoading ? "Analyse..." : "Rechercher" }}</NxButton>
      </div>
      <NxCard v-if="largeFilesError" danger>{{ largeFilesError }}</NxCard>
      <div v-for="f in largeFiles" :key="f.path" class="ft-row">
        <span>{{ f.path }}</span>
        <span>{{ bytesToMb(f.size_bytes) }} Mo</span>
      </div>
    </NxCard>

    <NxCard v-else>
      <div class="ft-form-row">
        <NxInput v-model="hashPath" placeholder="Chemin du fichier..." />
        <NxSelect v-model="hashAlgorithm" :options="HASH_OPTIONS" />
        <NxButton @click="computeHash">Calculer</NxButton>
      </div>
      <div class="ft-form-row">
        <NxInput v-model="expectedHash" placeholder="Hash attendu (ex: publié par l'éditeur)..." />
        <NxButton @click="verifyHash">Vérifier</NxButton>
      </div>
      <NxCard v-if="hashError" danger>{{ hashError }}</NxCard>
      <div v-if="hashResult" class="ft-hash-result">{{ hashResult }}</div>
      <NxBadge v-if="verifyMatch === true" status="success">Le hash correspond</NxBadge>
      <NxBadge v-else-if="verifyMatch === false" status="danger">Le hash ne correspond pas</NxBadge>
    </NxCard>
  </div>
</template>

<style scoped>
.ft-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.ft-tabs { display: flex; gap: 8px; }
.ft-tabs button { padding: 8px 14px; border-radius: var(--nx-style-radius); border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-secondary); cursor: pointer; font: inherit; }
.ft-tabs button.active { color: var(--nx-text-primary); font-weight: 600; }
.ft-form-row { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; }
.ft-dup-group { font-size: 13px; padding: 8px 0; }
.ft-row { display: flex; justify-content: space-between; padding: 6px 0; font-size: 13px; }
.ft-hash-result { font-family: monospace; font-size: 12px; word-break: break-all; padding-top: 8px; }
</style>
