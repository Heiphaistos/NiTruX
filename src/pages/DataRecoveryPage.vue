<!-- src/pages/DataRecoveryPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface TrashedItem { trashed_name: string; original_path: string; deletion_date: string }

const items = ref<TrashedItem[] | null>(null);
const error = ref<string | null>(null);
const busy = ref<string | null>(null);

async function refresh() {
  items.value = await invoke<TrashedItem[]>("list_trash");
}

onMounted(refresh);

function removeFromList(trashedName: string) {
  items.value = (items.value ?? []).filter((i) => i.trashed_name !== trashedName);
}

async function restore(trashedName: string) {
  busy.value = trashedName;
  error.value = null;
  try {
    await invoke("restore_trash_item", { trashedName });
    removeFromList(trashedName);
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = null;
  }
}

// "Supprimer définitivement" is genuinely irreversible -- unlike restore,
// there is no way back once this succeeds. Mirrors the type-to-confirm
// pattern already established elsewhere in this app for comparably
// irreversible actions (UninstallerPage's uninstall, DisksPage's
// format-partition) -- a single accidental click previously destroyed
// data with zero confirmation step at all.
const confirmingDelete = ref<string | null>(null);
const confirmDeleteText = ref("");

function startConfirmDelete(trashedName: string) {
  confirmingDelete.value = trashedName;
  confirmDeleteText.value = "";
}

async function deletePermanently(trashedName: string) {
  busy.value = trashedName;
  error.value = null;
  try {
    await invoke("delete_trash_item_permanently", { trashedName });
    removeFromList(trashedName);
    confirmingDelete.value = null;
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = null;
  }
}
</script>

<template>
  <div class="dr-page">
    <NxSectionHeader title="Récupération de données" description="Corbeille — restaurez un fichier récemment supprimé ou effacez-le définitivement." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <div v-if="items && items.length === 0" class="dr-empty">Corbeille vide.</div>

    <NxCard v-for="item in items ?? []" :key="item.trashed_name" class="dr-row">
      <div class="dr-row-top">
        <div class="dr-info">
          <span>{{ item.original_path }}</span>
          <span class="dr-date">{{ item.deletion_date }}</span>
        </div>
        <div class="dr-actions">
          <NxButton :disabled="busy !== null" @click="restore(item.trashed_name)">
            {{ busy === item.trashed_name ? "..." : "Restaurer" }}
          </NxButton>
          <NxButton v-if="confirmingDelete !== item.trashed_name" variant="danger" :disabled="busy !== null" @click="startConfirmDelete(item.trashed_name)">
            Supprimer définitivement
          </NxButton>
        </div>
      </div>
      <div v-if="confirmingDelete === item.trashed_name" class="dr-confirm-row">
        <NxInput
          v-model="confirmDeleteText"
          :placeholder="`Tapez « ${item.trashed_name} » pour confirmer`"
          aria-label="Confirmation du nom du fichier à supprimer définitivement"
        />
        <NxButton
          variant="danger"
          :disabled="busy !== null || confirmDeleteText !== item.trashed_name"
          @click="deletePermanently(item.trashed_name)"
        >
          {{ busy === item.trashed_name ? "Suppression..." : "Confirmer la suppression" }}
        </NxButton>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.dr-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.dr-empty { color: var(--nx-text-secondary); }
.dr-row { display: flex; flex-direction: column; gap: 10px; }
.dr-row-top { display: flex; justify-content: space-between; align-items: center; gap: 10px; }
.dr-info { display: flex; flex-direction: column; gap: 4px; font-size: 13px; }
.dr-date { color: var(--nx-text-secondary); font-size: 11px; }
.dr-actions { display: flex; gap: 8px; }
.dr-confirm-row { display: flex; gap: 10px; align-items: center; }
</style>
