<!-- src/pages/DisksPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSelect from "@/components/ui/NxSelect.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface Partition { name: string; size: string; mountpoint: string | null }
interface Disk { name: string; size: string; partitions: Partition[] }
interface UsageEntry { mountpoint: string; total_bytes: number; used_bytes: number; used_percent: number }

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

interface SmartStatus { device: string; health: string | null }

const smartStatus = ref<Record<string, SmartStatus>>({});
const smartError = ref<Record<string, string>>({});
// Per-disk, not a single shared name: a single ref would be overwritten by
// whichever disk starts checking most recently, so an earlier disk's
// `finally` could clear "busy" state that might now belong to a different,
// still-checking disk -- same class of bug already fixed for
// SystemToolsPage.vue's `running` ref.
const smartBusy = ref<Record<string, boolean>>({});

async function checkSmart(diskName: string) {
  const device = `/dev/${diskName}`;
  smartBusy.value = { ...smartBusy.value, [diskName]: true };
  delete smartError.value[diskName];
  try {
    smartStatus.value = { ...smartStatus.value, [diskName]: await invoke<SmartStatus>("get_smart_status", { device }) };
  } catch (e) {
    smartError.value = { ...smartError.value, [diskName]: String(e) };
  } finally {
    const { [diskName]: _removed, ...rest } = smartBusy.value;
    smartBusy.value = rest;
  }
}

const FSTYPE_OPTIONS = [
  { value: "ext4", label: "ext4" },
  { value: "btrfs", label: "btrfs" },
  { value: "xfs", label: "xfs" },
  { value: "vfat", label: "vfat" },
];

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

function bytesToGb(bytes: number): string {
  return (bytes / 1024 / 1024 / 1024).toFixed(1);
}
</script>

<template>
  <div class="disks-page">
    <NxSectionHeader title="Disques & partitions" description="État des disques, formatage, extension et clonage." />

    <NxCard v-if="disksError" danger>{{ disksError }}</NxCard>

    <NxCard v-for="disk in disks" :key="disk.name" class="disks-disk-card">
      <strong>{{ disk.name }}</strong> — {{ disk.size }}
      <ul>
        <li v-for="p in disk.partitions" :key="p.name">{{ p.name }} ({{ p.size }}){{ p.mountpoint ? ` → ${p.mountpoint}` : "" }}</li>
      </ul>
      <div class="disks-smart-row">
        <NxButton :disabled="smartBusy[disk.name] === true" @click="checkSmart(disk.name)">
          {{ smartBusy[disk.name] === true ? "Vérification..." : "Vérifier la santé" }}
        </NxButton>
        <NxBadge v-if="smartStatus[disk.name]" :status="smartStatus[disk.name].health === 'PASSED' ? 'success' : 'danger'">
          {{ smartStatus[disk.name].health ?? "inconnu" }}
        </NxBadge>
        <span v-if="smartError[disk.name]" class="disks-smart-error">{{ smartError[disk.name] }}</span>
      </div>
    </NxCard>
    <div v-if="!disksError && disks.length === 0" class="disks-empty">Aucun disque détecté.</div>

    <NxCard>
      <div v-for="u in usage" :key="u.mountpoint" class="disks-usage-row">
        <span>{{ u.mountpoint }}</span>
        <span>{{ bytesToGb(u.used_bytes) }} / {{ bytesToGb(u.total_bytes) }} Go ({{ u.used_percent }}%)</span>
      </div>
      <div v-if="!disksError && usage.length === 0" class="disks-empty">Aucune information d'utilisation disque.</div>
    </NxCard>

    <NxCard staticDanger>
      <NxSectionHeader title="Formater une partition" description="Cette action efface DÉFINITIVEMENT toutes les données de la partition. Aucune récupération possible." />
      <div class="disks-form-row">
        <NxInput v-model="formatDevice" placeholder="Périphérique (ex: /dev/sda1)" aria-label="Périphérique à formater" />
        <NxSelect v-model="formatFstype" :options="FSTYPE_OPTIONS" aria-label="Système de fichiers" />
      </div>
      <div class="disks-form-row">
        <NxInput
          v-model="formatConfirmText"
          :placeholder="`Tapez « ${formatDevice} » pour confirmer`"
          aria-label="Confirmation du périphérique à formater"
        />
        <NxButton
          variant="danger"
          :disabled="formatBusy || formatDevice === '' || formatConfirmText !== formatDevice"
          @click="runFormat"
        >
          {{ formatBusy ? "Formatage..." : "Formater" }}
        </NxButton>
      </div>
      <NxCard v-if="formatError" danger>{{ formatError }}</NxCard>
      <NxBadge v-if="formatResult" status="success" live>{{ formatResult }}</NxBadge>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Étendre une partition" />
      <div class="disks-form-row">
        <NxInput v-model="extendDevice" placeholder="Partition (ex: /dev/sda1)" aria-label="Partition à étendre" />
        <NxInput v-model="extendDisk" placeholder="Disque (ex: /dev/sda)" aria-label="Disque contenant la partition" />
        <NxInput v-model="extendPartNumber" placeholder="N° (ex: 1)" aria-label="Numéro de la partition" />
        <NxButton :disabled="extendBusy" @click="runExtend">{{ extendBusy ? "Extension..." : "Étendre" }}</NxButton>
      </div>
      <NxCard v-if="extendError" danger>{{ extendError }}</NxCard>
      <NxBadge v-if="extendResult" status="success" live>{{ extendResult }}</NxBadge>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Cloner un disque" />
      <div class="disks-form-row">
        <NxInput v-model="cloneSourceDisk" placeholder="Disque source (ex: /dev/sda)" aria-label="Disque source à cloner" />
        <NxInput v-model="cloneDestPath" placeholder="Fichier image de destination" aria-label="Chemin du fichier image de destination" />
        <NxButton :disabled="cloneBusy" @click="runClone">{{ cloneBusy ? "Clonage..." : "Cloner" }}</NxButton>
      </div>
      <NxCard v-if="cloneError" danger>{{ cloneError }}</NxCard>
      <NxBadge v-if="cloneResult" status="success" live>{{ cloneResult }}</NxBadge>
    </NxCard>
  </div>
</template>

<style scoped>
.disks-page { padding: 24px; display: flex; flex-direction: column; gap: 14px; }
.disks-empty { color: var(--nx-text-secondary); font-size: 13px; }
.disks-disk-card { font-size: 13px; }
.disks-usage-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 4px 0; font-size: 13px; }
.disks-form-row { display: flex; gap: 10px; align-items: center; margin: 10px 0; }
.disks-smart-row { display: flex; align-items: center; gap: 10px; margin-top: 8px; font-size: 12px; }
.disks-smart-error { color: var(--nx-text-secondary); }
</style>
