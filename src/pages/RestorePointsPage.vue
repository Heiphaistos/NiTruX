<!-- src/pages/RestorePointsPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface SnapshotInfo { id: string; date: string }

const snapshots = ref<SnapshotInfo[]>([]);
const snapshotsError = ref<string | null>(null);

async function loadSnapshots() {
  snapshotsError.value = null;
  try {
    snapshots.value = await invoke<SnapshotInfo[]>("list_snapshots");
  } catch (e) {
    snapshotsError.value = String(e);
  }
}

onMounted(loadSnapshots);

const snapshotCreating = ref(false);
const snapshotCreateError = ref<string | null>(null);

async function createSnapshotNow() {
  snapshotCreating.value = true;
  snapshotCreateError.value = null;
  try {
    await invoke("create_snapshot");
    await loadSnapshots();
  } catch (e) {
    snapshotCreateError.value = String(e);
  } finally {
    snapshotCreating.value = false;
  }
}
</script>

<template>
  <div class="rp-page">
    <NxSectionHeader title="Restauration" description="Instantanés système." />

    <NxCard>
      <NxButton :disabled="snapshotCreating" @click="createSnapshotNow">{{ snapshotCreating ? "Création..." : "Créer un instantané" }}</NxButton>
    </NxCard>

    <NxCard v-if="snapshotCreateError" danger>{{ snapshotCreateError }}</NxCard>
    <NxCard v-if="snapshotsError" danger>{{ snapshotsError }}</NxCard>

    <NxCard v-for="s in snapshots" :key="s.id" class="rp-row">
      <span>#{{ s.id }}</span>
      <span>{{ s.date }}</span>
    </NxCard>
    <NxCard v-if="!snapshotsError && snapshots.length === 0" class="rp-empty">Aucun instantané trouvé.</NxCard>
  </div>
</template>

<style scoped>
.rp-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.rp-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; font-size: 13px; }
.rp-empty { color: var(--nx-text-secondary); font-size: 13px; }
</style>
