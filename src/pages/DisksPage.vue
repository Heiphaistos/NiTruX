<!-- src/pages/DisksPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface Partition { name: string; size: string; mountpoint: string | null }
interface Disk { name: string; size: string; partitions: Partition[] }
interface UsageEntry { mountpoint: string; total_bytes: number; used_bytes: number; used_percent: number }
interface DuplicateGroup { hash: string; paths: string[]; size_bytes: number }
interface LargeFile { path: string; size_bytes: number }

type Tab = "disks" | "duplicates" | "largefiles" | "hashcheck";
const activeTab = ref<Tab>("disks");

const disks = ref<Disk[]>([]);
const usage = ref<UsageEntry[]>([]);
const disksError = ref<string | null>(null);

async function loadDisks() {
  disksError.value = null;
  try {
    disks.value = await invoke<Disk[]>("list_disks");
    usage.value = await invoke<UsageEntry[]>("list_disk_usage");
  } catch (e) {
    disksError.value = String(e);
  }
}
onMounted(loadDisks);

const scanDir = ref("");
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

const largeFileDir = ref("");
const minSizeMb = ref(100);
const largeFiles = ref<LargeFile[]>([]);
const largeFilesError = ref<string | null>(null);
const largeFilesLoading = ref(false);

async function scanLargeFiles() {
  largeFilesLoading.value = true;
  largeFilesError.value = null;
  try {
    largeFiles.value = await invoke<LargeFile[]>("find_large_files_cmd", {
      directory: largeFileDir.value,
      minSizeBytes: minSizeMb.value * 1024 * 1024,
    });
  } catch (e) {
    largeFilesError.value = String(e);
  } finally {
    largeFilesLoading.value = false;
  }
}

const hashPath = ref("");
const hashAlgorithm = ref<"sha256" | "sha1" | "md5">("sha256");
const hashResult = ref<string | null>(null);
const hashError = ref<string | null>(null);

async function computeHash() {
  hashError.value = null;
  hashResult.value = null;
  try {
    hashResult.value = await invoke<string>("compute_file_hash", { path: hashPath.value, algorithm: hashAlgorithm.value });
  } catch (e) {
    hashError.value = String(e);
  }
}

function bytesToGb(bytes: number): string {
  return (bytes / 1024 / 1024 / 1024).toFixed(1);
}

function bytesToMb(bytes: number): string {
  return (bytes / 1024 / 1024).toFixed(1);
}
</script>

<template>
  <div class="disks-page">
    <h1>Disques & stockage</h1>

    <div class="disks-tabs">
      <button :class="{ active: activeTab === 'disks' }" @click="activeTab = 'disks'">Disques</button>
      <button :class="{ active: activeTab === 'duplicates' }" @click="activeTab = 'duplicates'">Doublons</button>
      <button :class="{ active: activeTab === 'largefiles' }" @click="activeTab = 'largefiles'">Gros fichiers</button>
      <button :class="{ active: activeTab === 'hashcheck' }" @click="activeTab = 'hashcheck'">Vérif. hash</button>
    </div>

    <section v-if="activeTab === 'disks'" class="disks-panel">
      <div v-if="disksError" class="disks-error">{{ disksError }}</div>
      <div v-for="disk in disks" :key="disk.name" class="disks-disk-card">
        <strong>{{ disk.name }}</strong> — {{ disk.size }}
        <ul>
          <li v-for="p in disk.partitions" :key="p.name">{{ p.name }} ({{ p.size }}){{ p.mountpoint ? ` → ${p.mountpoint}` : "" }}</li>
        </ul>
      </div>
      <div v-for="u in usage" :key="u.mountpoint" class="disks-usage-row">
        <span>{{ u.mountpoint }}</span>
        <span>{{ bytesToGb(u.used_bytes) }} / {{ bytesToGb(u.total_bytes) }} GB ({{ u.used_percent }}%)</span>
      </div>
    </section>

    <section v-else-if="activeTab === 'duplicates'" class="disks-panel">
      <div class="disks-form-row">
        <input v-model="scanDir" class="disks-input" placeholder="Dossier à scanner..." />
        <button :disabled="duplicatesLoading" @click="scanDuplicates">{{ duplicatesLoading ? "Analyse..." : "Rechercher" }}</button>
      </div>
      <div v-if="duplicatesError" class="disks-error">{{ duplicatesError }}</div>
      <div v-for="g in duplicateGroups" :key="g.hash" class="disks-dup-group">
        <div>{{ g.paths.length }} fichiers identiques ({{ bytesToMb(g.size_bytes) }} MB chacun)</div>
        <ul><li v-for="p in g.paths" :key="p">{{ p }}</li></ul>
      </div>
    </section>

    <section v-else-if="activeTab === 'largefiles'" class="disks-panel">
      <div class="disks-form-row">
        <input v-model="largeFileDir" class="disks-input" placeholder="Dossier à scanner..." />
        <input v-model.number="minSizeMb" type="number" class="disks-input-small" /> MB min
        <button :disabled="largeFilesLoading" @click="scanLargeFiles">{{ largeFilesLoading ? "Analyse..." : "Rechercher" }}</button>
      </div>
      <div v-if="largeFilesError" class="disks-error">{{ largeFilesError }}</div>
      <div v-for="f in largeFiles" :key="f.path" class="disks-usage-row">
        <span>{{ f.path }}</span>
        <span>{{ bytesToMb(f.size_bytes) }} MB</span>
      </div>
    </section>

    <section v-else class="disks-panel">
      <div class="disks-form-row">
        <input v-model="hashPath" class="disks-input" placeholder="Chemin du fichier..." />
        <select v-model="hashAlgorithm">
          <option value="sha256">SHA-256</option>
          <option value="sha1">SHA-1</option>
          <option value="md5">MD5</option>
        </select>
        <button @click="computeHash">Calculer</button>
      </div>
      <div v-if="hashError" class="disks-error">{{ hashError }}</div>
      <div v-if="hashResult" class="disks-hash-result">{{ hashResult }}</div>
    </section>
  </div>
</template>

<style scoped>
.disks-page { padding: 24px; color: var(--nx-text-primary); }
.disks-tabs { display: flex; gap: 8px; margin: 16px 0; }
.disks-tabs button { padding: 8px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-secondary); cursor: pointer; }
.disks-tabs button.active { color: var(--nx-text-primary); border-color: var(--nx-accent-primary); }
.disks-panel { display: flex; flex-direction: column; gap: 10px; }
.disks-error { padding: 10px 14px; border-radius: 8px; background: color-mix(in srgb, var(--nx-accent-danger) 15%, transparent); border: 1px solid var(--nx-accent-danger); }
.disks-disk-card { background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); border-radius: 10px; padding: 12px; }
.disks-usage-row, .disks-form-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; }
.disks-input { flex: 1; padding: 8px 10px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); }
.disks-input-small { width: 80px; padding: 8px 10px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); }
.disks-dup-group { background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); border-radius: 10px; padding: 12px; font-size: 13px; }
.disks-hash-result { font-family: monospace; padding: 10px 14px; border-radius: 8px; background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); word-break: break-all; }
</style>
