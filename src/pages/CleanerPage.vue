<!-- src/pages/CleanerPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxStatTile from "@/components/ui/NxStatTile.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface CacheSizeReport { user_cache_bytes: number; package_cache_bytes: number | null }

function formatMb(bytes: number): string {
  return `${(bytes / 1_048_576).toFixed(1)} Mo`;
}

const report = ref<CacheSizeReport | null>(null);
const reportError = ref<string | null>(null);

async function loadReport() {
  try {
    report.value = await invoke<CacheSizeReport>("get_cache_size_report");
  } catch (e) {
    reportError.value = String(e);
  }
}

onMounted(loadReport);

const busy = ref<string | null>(null);
const result = ref<string | null>(null);
const actionError = ref<string | null>(null);

async function runAction(action: "clean-cache" | "vacuum-logs") {
  busy.value = action;
  actionError.value = null;
  result.value = null;
  try {
    result.value = await invoke<string>("run_troubleshoot_action", { action });
    if (action === "clean-cache") await loadReport();
  } catch (e) {
    actionError.value = String(e);
  } finally {
    busy.value = null;
  }
}
</script>

<template>
  <div class="cln-page">
    <NxSectionHeader title="Nettoyeur" description="Aperçu de l'espace utilisé par les caches et purge des fichiers temporaires." />

    <NxCard v-if="reportError" danger>{{ reportError }}</NxCard>

    <div class="cln-stats" v-if="report">
      <NxCard><NxStatTile label="Cache utilisateur (~/.cache)" :value="formatMb(report.user_cache_bytes)" /></NxCard>
      <NxCard>
        <NxStatTile label="Cache du gestionnaire de paquets" :value="report.package_cache_bytes !== null ? formatMb(report.package_cache_bytes) : 'inconnu'" />
      </NxCard>
    </div>

    <NxCard>
      <NxCard v-if="actionError" danger>{{ actionError }}</NxCard>
      <NxBadge v-if="result" status="success">{{ result }}</NxBadge>
      <div class="cln-action-row">
        <span class="cln-action-label">Cache des paquets</span>
        <NxButton :disabled="busy !== null" @click="runAction('clean-cache')">{{ busy === "clean-cache" ? "En cours..." : "Vider le cache" }}</NxButton>
      </div>
      <div class="cln-action-row">
        <span class="cln-action-label">Anciens journaux</span>
        <NxButton :disabled="busy !== null" @click="runAction('vacuum-logs')">{{ busy === "vacuum-logs" ? "En cours..." : "Purger les journaux" }}</NxButton>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.cln-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.cln-stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 12px; }
.cln-action-row { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; }
.cln-action-label { flex: 1; }
</style>
