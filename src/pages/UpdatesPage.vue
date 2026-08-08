<!-- src/pages/UpdatesPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface PackageUpdate {
  name: string;
  current_version: string;
  new_version: string;
  source: string;
}

const updates = ref<PackageUpdate[]>([]);
const error = ref<string | null>(null);
const loading = ref(false);

async function refresh() {
  loading.value = true;
  error.value = null;
  try {
    updates.value = await invoke<PackageUpdate[]>("list_updates");
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(refresh);

const upgrading = ref(false);
const upgradeResult = ref<string | null>(null);
const upgradeError = ref<string | null>(null);

// "Vérifier" is also disabled while an upgrade is running (not just its
// own `loading`): upgradeAll() already calls refresh() internally once the
// upgrade finishes -- a manual click mid-upgrade would race a second
// refresh() against that one, both writing `updates`/`loading`
// concurrently with no guarantee which write lands last.
async function upgradeAll() {
  upgrading.value = true;
  upgradeError.value = null;
  upgradeResult.value = null;
  try {
    upgradeResult.value = await invoke<string>("upgrade_all_packages");
    await refresh();
  } catch (e) {
    upgradeError.value = String(e);
  } finally {
    upgrading.value = false;
  }
}
</script>

<template>
  <div class="upd-page">
    <div class="upd-header">
      <NxSectionHeader title="Mises à jour" description="Paquets pouvant être mis à jour, tous gestionnaires détectés confondus." />
      <div class="upd-actions">
        <NxButton :disabled="loading || upgrading" @click="refresh">{{ loading ? "Vérification..." : "Vérifier" }}</NxButton>
        <NxButton :disabled="upgrading || updates.length === 0" @click="upgradeAll">
          {{ upgrading ? "Mise à jour..." : "Tout mettre à jour" }}
        </NxButton>
      </div>
    </div>

    <NxCard v-if="upgradeError" danger>{{ upgradeError }}</NxCard>
    <NxBadge v-if="upgradeResult" status="success" live>Mise à jour terminée.</NxBadge>

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <div v-else-if="!loading && updates.length === 0" class="upd-empty">
      Aucune mise à jour disponible.
    </div>

    <NxCard v-else>
      <div class="upd-table-scroll">
        <table class="upd-table">
          <thead>
            <tr><th>Source</th><th>Paquet</th><th>Version actuelle</th><th>Nouvelle version</th></tr>
          </thead>
          <tbody>
            <tr v-for="u in updates" :key="`${u.source}-${u.name}`">
              <td><NxBadge status="info">{{ u.source }}</NxBadge></td>
              <td>{{ u.name }}</td>
              <td>{{ u.current_version || "—" }}</td>
              <td>{{ u.new_version }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.upd-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.upd-header { display: flex; justify-content: space-between; align-items: flex-start; gap: 12px; flex-wrap: wrap; }
.upd-actions { display: flex; gap: 10px; }
.upd-empty { color: var(--nx-text-secondary); }
.upd-table-scroll { overflow-x: auto; }
.upd-table { width: 100%; min-width: 480px; border-collapse: collapse; font-size: 13px; }
.upd-table th { text-align: left; color: var(--nx-text-secondary); border-bottom: 1px solid var(--nx-style-border-color); padding: 8px; }
.upd-table td { padding: 8px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
