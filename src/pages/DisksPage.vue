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

const formatDevice = ref("");
const formatFstype = ref<"ext4" | "btrfs" | "xfs" | "vfat">("ext4");
const formatConfirmText = ref("");
const formatBusy = ref(false);
const formatResult = ref<string | null>(null);
const formatError = ref<string | null>(null);

async function runFormat() {
  formatBusy.value = true;
  formatError.value = null;
  formatResult.value = null;
  try {
    formatResult.value = await invoke<string>("format_partition", { device: formatDevice.value, fstype: formatFstype.value });
    formatConfirmText.value = "";
    await loadDisks();
  } catch (e) {
    formatError.value = String(e);
  } finally {
    formatBusy.value = false;
  }
}

const extendDevice = ref("");
const extendDisk = ref("");
const extendPartNumber = ref("");
const extendBusy = ref(false);
const extendResult = ref<string | null>(null);
const extendError = ref<string | null>(null);

async function runExtend() {
  extendBusy.value = true;
  extendError.value = null;
  extendResult.value = null;
  try {
    extendResult.value = await invoke<string>("extend_partition", {
      device: extendDevice.value,
      disk: extendDisk.value,
      partNumber: extendPartNumber.value,
    });
    await loadDisks();
  } catch (e) {
    extendError.value = String(e);
  } finally {
    extendBusy.value = false;
  }
}

const cloneSourceDisk = ref("");
const cloneDestPath = ref("");
const cloneBusy = ref(false);
const cloneResult = ref<string | null>(null);
const cloneError = ref<string | null>(null);

async function runClone() {
  cloneBusy.value = true;
  cloneError.value = null;
  cloneResult.value = null;
  try {
    cloneResult.value = await invoke<string>("clone_disk", { sourceDisk: cloneSourceDisk.value, destPath: cloneDestPath.value });
  } catch (e) {
    cloneError.value = String(e);
  } finally {
    cloneBusy.value = false;
  }
}

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

      <h2 class="disks-section-title">Formater une partition</h2>
      <p class="disks-warning">Cette action efface DÉFINITIVEMENT toutes les données de la partition. Aucune récupération possible.</p>
      <div class="disks-form-row">
        <input v-model="formatDevice" class="disks-input" placeholder="Périphérique (ex: /dev/sda1)" />
        <select v-model="formatFstype">
          <option value="ext4">ext4</option>
          <option value="btrfs">btrfs</option>
          <option value="xfs">xfs</option>
          <option value="vfat">vfat</option>
        </select>
      </div>
      <div class="disks-form-row">
        <input
          v-model="formatConfirmText"
          class="disks-input"
          :placeholder="`Tapez « ${formatDevice} » pour confirmer`"
        />
        <button
          :disabled="formatBusy || formatDevice === '' || formatConfirmText !== formatDevice"
          class="disks-danger-button"
          @click="runFormat"
        >
          {{ formatBusy ? "Formatage..." : "Formater" }}
        </button>
      </div>
      <div v-if="formatError" class="disks-error">{{ formatError }}</div>
      <div v-if="formatResult" class="disks-success">{{ formatResult }}</div>

      <h2 class="disks-section-title">Étendre une partition</h2>
      <div class="disks-form-row">
        <input v-model="extendDevice" class="disks-input" placeholder="Partition (ex: /dev/sda1)" />
        <input v-model="extendDisk" class="disks-input" placeholder="Disque (ex: /dev/sda)" />
        <input v-model="extendPartNumber" class="disks-input-small" placeholder="N° (ex: 1)" />
        <button :disabled="extendBusy" @click="runExtend">{{ extendBusy ? "Extension..." : "Étendre" }}</button>
      </div>
      <div v-if="extendError" class="disks-error">{{ extendError }}</div>
      <div v-if="extendResult" class="disks-success">{{ extendResult }}</div>

      <h2 class="disks-section-title">Cloner un disque</h2>
      <div class="disks-form-row">
        <input v-model="cloneSourceDisk" class="disks-input" placeholder="Disque source (ex: /dev/sda)" />
        <input v-model="cloneDestPath" class="disks-input" placeholder="Fichier image de destination" />
        <button :disabled="cloneBusy" @click="runClone">{{ cloneBusy ? "Clonage..." : "Cloner" }}</button>
      </div>
      <div v-if="cloneError" class="disks-error">{{ cloneError }}</div>
      <div v-if="cloneResult" class="disks-success">{{ cloneResult }}</div>
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
.disks-section-title { font-size: 14px; margin: 16px 0 6px; color: var(--nx-text-secondary); }
.disks-warning { padding: 10px 14px; border-radius: 8px; background: color-mix(in srgb, var(--nx-accent-danger) 10%, transparent); border: 1px solid var(--nx-accent-danger); color: var(--nx-accent-danger); font-weight: 600; font-size: 13px; }
.disks-danger-button { background: var(--nx-accent-danger); color: white; border-color: var(--nx-accent-danger); }
.disks-success { padding: 10px 14px; border-radius: 8px; background: color-mix(in srgb, var(--nx-accent-success) 15%, transparent); border: 1px solid var(--nx-accent-success); }
</style>
