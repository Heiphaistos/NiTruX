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

interface OrphanConfig { path: string; name: string; size_bytes: number; kind: string }

const orphans = ref<OrphanConfig[] | null>(null);
const orphansError = ref<string | null>(null);
const scanningOrphans = ref(false);
const trashing = ref<Record<string, boolean>>({});
const trashError = ref<string | null>(null);

async function scanOrphans() {
  scanningOrphans.value = true;
  orphansError.value = null;
  try {
    orphans.value = await invoke<OrphanConfig[]>("list_orphan_configs");
  } catch (e) {
    orphansError.value = String(e);
  } finally {
    scanningOrphans.value = false;
  }
}

// To the trash, never unlinked: "orphan" is a heuristic, and a wrong guess
// must stay undoable from the user's own file manager.
async function trashOrphan(orphan: OrphanConfig) {
  trashing.value = { ...trashing.value, [orphan.path]: true };
  trashError.value = null;
  try {
    await invoke<string>("move_to_trash", { path: orphan.path });
    orphans.value = (orphans.value ?? []).filter((o) => o.path !== orphan.path);
  } catch (e) {
    trashError.value = String(e);
  } finally {
    trashing.value = { ...trashing.value, [orphan.path]: false };
  }
}

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
      <NxBadge v-if="result" status="success" live>{{ result }}</NxBadge>
      <div class="cln-action-row">
        <span class="cln-action-label">Cache des paquets</span>
        <NxButton :disabled="busy !== null" @click="runAction('clean-cache')">{{ busy === "clean-cache" ? "En cours..." : "Vider le cache" }}</NxButton>
      </div>
      <div class="cln-action-row">
        <span class="cln-action-label">Anciens journaux</span>
        <NxButton :disabled="busy !== null" @click="runAction('vacuum-logs')">{{ busy === "vacuum-logs" ? "En cours..." : "Purger les journaux" }}</NxButton>
      </div>
    </NxCard>

    <NxCard>
      <NxSectionHeader
        title="Configurations orphelines"
        description="Dossiers de ~/.config, ~/.local/share et ~/.cache dont l'application ne semble plus installée. Désinstaller un paquet ne supprime jamais ces fichiers."
      />
      <div class="cln-action-row">
        <span class="cln-action-label">
          {{ orphans ? `${orphans.length} dossier(s) sans application correspondante` : "Analyse non lancée" }}
        </span>
        <NxButton :disabled="scanningOrphans" @click="scanOrphans">
          {{ scanningOrphans ? "Analyse..." : "Analyser" }}
        </NxButton>
      </div>

      <NxCard v-if="orphansError" danger>{{ orphansError }}</NxCard>
      <NxCard v-if="trashError" danger>{{ trashError }}</NxCard>
      <div v-if="orphans && orphans.length === 0" class="cln-empty">
        Aucun dossier orphelin détecté.
      </div>

      <div v-for="o in orphans ?? []" :key="o.path" class="cln-orphan-row">
        <span class="cln-orphan-name">{{ o.name }}</span>
        <span class="cln-orphan-meta">{{ o.kind }} · {{ formatMb(o.size_bytes) }}</span>
        <NxButton :disabled="trashing[o.path]" @click="trashOrphan(o)">
          {{ trashing[o.path] ? "…" : "Mettre à la corbeille" }}
        </NxButton>
      </div>
      <p v-if="orphans && orphans.length > 0" class="cln-note">
        Détection heuristique : rien n'est supprimé, tout part à la corbeille et reste restaurable.
      </p>
    </NxCard>
  </div>
</template>

<style scoped>
.cln-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.cln-stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 12px; }
.cln-action-row { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; }
.cln-action-label { flex: 1; }
.cln-empty { color: var(--nx-text-secondary); font-size: 13px; }
.cln-orphan-row { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
.cln-orphan-name { flex: 1; min-width: 140px; font-family: monospace; word-break: break-all; }
.cln-orphan-meta { color: var(--nx-text-secondary); }
.cln-note { margin: 10px 0 0; font-size: 12px; color: var(--nx-text-secondary); }
</style>
