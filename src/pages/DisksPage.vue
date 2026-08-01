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
    </NxCard>

    <NxCard>
      <div v-for="u in usage" :key="u.mountpoint" class="disks-usage-row">
        <span>{{ u.mountpoint }}</span>
        <span>{{ bytesToGb(u.used_bytes) }} / {{ bytesToGb(u.total_bytes) }} GB ({{ u.used_percent }}%)</span>
      </div>
    </NxCard>

    <NxCard danger>
      <NxSectionHeader title="Formater une partition" description="Cette action efface DÉFINITIVEMENT toutes les données de la partition. Aucune récupération possible." />
      <div class="disks-form-row">
        <NxInput v-model="formatDevice" placeholder="Périphérique (ex: /dev/sda1)" />
        <NxSelect v-model="formatFstype" :options="FSTYPE_OPTIONS" />
      </div>
      <div class="disks-form-row">
        <NxInput
          v-model="formatConfirmText"
          :placeholder="`Tapez « ${formatDevice} » pour confirmer`"
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
      <NxBadge v-if="formatResult" status="success">{{ formatResult }}</NxBadge>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Étendre une partition" />
      <div class="disks-form-row">
        <NxInput v-model="extendDevice" placeholder="Partition (ex: /dev/sda1)" />
        <NxInput v-model="extendDisk" placeholder="Disque (ex: /dev/sda)" />
        <NxInput v-model="extendPartNumber" placeholder="N° (ex: 1)" />
        <NxButton :disabled="extendBusy" @click="runExtend">{{ extendBusy ? "Extension..." : "Étendre" }}</NxButton>
      </div>
      <NxCard v-if="extendError" danger>{{ extendError }}</NxCard>
      <NxBadge v-if="extendResult" status="success">{{ extendResult }}</NxBadge>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Cloner un disque" />
      <div class="disks-form-row">
        <NxInput v-model="cloneSourceDisk" placeholder="Disque source (ex: /dev/sda)" />
        <NxInput v-model="cloneDestPath" placeholder="Fichier image de destination" />
        <NxButton :disabled="cloneBusy" @click="runClone">{{ cloneBusy ? "Clonage..." : "Cloner" }}</NxButton>
      </div>
      <NxCard v-if="cloneError" danger>{{ cloneError }}</NxCard>
      <NxBadge v-if="cloneResult" status="success">{{ cloneResult }}</NxBadge>
    </NxCard>
  </div>
</template>

<style scoped>
.disks-page { padding: 24px; display: flex; flex-direction: column; gap: 14px; }
.disks-disk-card { font-size: 13px; }
.disks-usage-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 4px 0; font-size: 13px; }
.disks-form-row { display: flex; gap: 10px; align-items: center; margin: 10px 0; }
</style>
