<!-- src/pages/DataRecoveryPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
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

async function deletePermanently(trashedName: string) {
  busy.value = trashedName;
  error.value = null;
  try {
    await invoke("delete_trash_item_permanently", { trashedName });
    removeFromList(trashedName);
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
      <div class="dr-info">
        <span>{{ item.original_path }}</span>
        <span class="dr-date">{{ item.deletion_date }}</span>
      </div>
      <div class="dr-actions">
        <NxButton :disabled="busy !== null" @click="restore(item.trashed_name)">
          {{ busy === item.trashed_name ? "..." : "Restaurer" }}
        </NxButton>
        <NxButton variant="danger" :disabled="busy !== null" @click="deletePermanently(item.trashed_name)">
          Supprimer définitivement
        </NxButton>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.dr-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.dr-empty { color: var(--nx-text-secondary); }
.dr-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; }
.dr-info { display: flex; flex-direction: column; gap: 4px; font-size: 13px; }
.dr-date { color: var(--nx-text-secondary); font-size: 11px; }
.dr-actions { display: flex; gap: 8px; }
</style>
