<!-- src/pages/RestorePointsPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface SnapshotInfo { id: string; date: string; handle: string }

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

// Snapper only lists for root: rather than a password prompt on every visit,
// listing waits for a click there. Timeshift lists as before.
const backend = ref<string>("timeshift");
const listed = ref(false);

onMounted(async () => {
  backend.value = await invoke<string>("get_snapshot_backend").catch(() => "timeshift");
  if (backend.value !== "snapper") {
    await loadSnapshots();
    listed.value = true;
  }
});

// Snapper only: one file back to its state in a snapshot. The helper keeps
// the current version as <file>.avant-restauration.
const restoreFor = ref<string | null>(null);
const restorePath = ref("");
const restoreBusy = ref(false);
const restoreResult = ref<string | null>(null);
const restoreError = ref<string | null>(null);

async function restoreFile(handle: string) {
  restoreBusy.value = true;
  restoreError.value = null;
  restoreResult.value = null;
  try {
    await invoke("restore_snapshot_file", { number: handle, path: restorePath.value });
    restoreResult.value = `${restorePath.value} remis dans l'état de l'instantané ${handle} (version actuelle gardée en .avant-restauration).`;
  } catch (e) {
    restoreError.value = String(e);
  } finally {
    restoreBusy.value = false;
  }
}

async function listSnapper() {
  await loadSnapshots();
  listed.value = !snapshotsError.value;
}

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
// delete-snapshot takes `s.handle`: Timeshift's timestamp name or Snapper's number,
// not the list index (s.id) shown next to it.
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
    <NxSectionHeader title="Restauration" :description="backend === 'snapper' ? 'Instantanés Snapper. Pour revenir à l\'un d\'eux, choisissez-le dans le menu de démarrage.' : 'Instantanés système (Timeshift).'" />

    <NxCard>
      <NxButton :disabled="snapshotCreating" @click="createSnapshotNow">{{ snapshotCreating ? "Création..." : "Créer un instantané" }}</NxButton>
      <NxButton v-if="backend === 'snapper' && !listed" variant="ghost" @click="listSnapper">Afficher les instantanés (mot de passe)</NxButton>
    </NxCard>

    <NxCard v-if="snapshotCreateError" danger>{{ snapshotCreateError }}</NxCard>
    <NxCard v-if="snapshotsError" danger>{{ snapshotsError }}</NxCard>
    <NxCard v-if="deleteError" danger>{{ deleteError }}</NxCard>
    <NxCard v-if="restoreError" danger>{{ restoreError }}</NxCard>
    <NxCard v-if="restoreResult">{{ restoreResult }}</NxCard>

    <NxCard v-for="s in snapshots" :key="s.id" class="rp-snapshot">
      <div class="rp-row">
        <span>#{{ s.id }}</span>
        <span>{{ s.date }}</span>
        <NxButton v-if="confirmingDelete !== s.handle" variant="danger" :disabled="deleting !== null" @click="startConfirmDelete(s.handle)">
          Supprimer
        </NxButton>
      </div>
      <div v-if="backend === 'snapper'" class="rp-confirm-row">
        <NxButton v-if="restoreFor !== s.handle" variant="ghost" @click="restoreFor = s.handle; restorePath = ''">Restaurer un fichier…</NxButton>
        <template v-else>
          <NxInput v-model="restorePath" placeholder="Chemin complet du fichier (ex : /etc/fstab)" :aria-label="`Fichier à restaurer depuis l'instantané ${s.handle}`" />
          <NxButton :disabled="restoreBusy || !restorePath.startsWith('/')" @click="restoreFile(s.handle)">{{ restoreBusy ? "Restauration…" : "Restaurer" }}</NxButton>
        </template>
      </div>
      <div v-if="confirmingDelete === s.handle" class="rp-confirm-row">
        <NxInput
          v-model="confirmDeleteText"
          :placeholder="`Tapez « ${s.handle} » pour confirmer`"
          :aria-label="`Confirmation de suppression de l'instantané ${s.date}`"
        />
        <NxButton
          variant="danger"
          :disabled="deleting !== null || confirmDeleteText !== s.handle"
          @click="deleteSnapshot(s.handle)"
        >
          {{ deleting === s.handle ? "Suppression..." : "Confirmer la suppression" }}
        </NxButton>
      </div>
    </NxCard>
    <NxCard v-if="listed && !snapshotsError && snapshots.length === 0" class="rp-empty">Aucun instantané trouvé.</NxCard>
  </div>
</template>

<style scoped>
.rp-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.rp-snapshot { display: flex; flex-direction: column; gap: 8px; }
.rp-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; font-size: 13px; }
.rp-confirm-row { display: flex; gap: 10px; align-items: center; }
.rp-empty { color: var(--nx-text-secondary); font-size: 13px; }
</style>
