<!-- src/pages/RestorePointsPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
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

// A deleted snapshot cannot be recovered, and it's a backup of a past
// system state -- the same irreversibility class as AntivirusPage's
// quarantine or DisksPage's format-partition, so it gets the same
// type-to-confirm pattern rather than a single unconfirmed click.
// timeshift's --delete needs the snapshot's timestamp NAME (s.date), not
// the numeric list index (s.id) shown next to it.
const confirmingDelete = ref<string | null>(null);
const confirmDeleteText = ref("");
const deleting = ref<string | null>(null);
const deleteError = ref<string | null>(null);

function startConfirmDelete(date: string) {
  confirmingDelete.value = date;
  confirmDeleteText.value = "";
}

async function deleteSnapshot(date: string) {
  deleting.value = date;
  deleteError.value = null;
  try {
    await invoke("delete_snapshot", { name: date });
    confirmingDelete.value = null;
    await loadSnapshots();
  } catch (e) {
    deleteError.value = String(e);
  } finally {
    deleting.value = null;
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
    <NxCard v-if="deleteError" danger>{{ deleteError }}</NxCard>

    <NxCard v-for="s in snapshots" :key="s.id" class="rp-snapshot">
      <div class="rp-row">
        <span>#{{ s.id }}</span>
        <span>{{ s.date }}</span>
        <NxButton v-if="confirmingDelete !== s.date" variant="danger" :disabled="deleting !== null" @click="startConfirmDelete(s.date)">
          Supprimer
        </NxButton>
      </div>
      <div v-if="confirmingDelete === s.date" class="rp-confirm-row">
        <NxInput
          v-model="confirmDeleteText"
          :placeholder="`Tapez « ${s.date} » pour confirmer`"
          :aria-label="`Confirmation de suppression de l'instantané ${s.date}`"
        />
        <NxButton
          variant="danger"
          :disabled="deleting !== null || confirmDeleteText !== s.date"
          @click="deleteSnapshot(s.date)"
        >
          {{ deleting === s.date ? "Suppression..." : "Confirmer la suppression" }}
        </NxButton>
      </div>
    </NxCard>
    <NxCard v-if="!snapshotsError && snapshots.length === 0" class="rp-empty">Aucun instantané trouvé.</NxCard>
  </div>
</template>

<style scoped>
.rp-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.rp-snapshot { display: flex; flex-direction: column; gap: 8px; }
.rp-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; font-size: 13px; }
.rp-confirm-row { display: flex; gap: 10px; align-items: center; }
.rp-empty { color: var(--nx-text-secondary); font-size: 13px; }
</style>
