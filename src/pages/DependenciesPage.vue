<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxButton from "@/components/ui/NxButton.vue";

interface MissingDependency { binary: string; missing_library: string }
interface ToolStatus { binary: string; package: string | null; feature: string; installed: boolean }

const results = ref<MissingDependency[] | null>(null);
const error = ref<string | null>(null);

const tools = ref<ToolStatus[] | null>(null);
const toolsError = ref<string | null>(null);
const nativeManager = ref<string | null>(null);
const installing = ref<Record<string, boolean>>({});
const installErrors = ref<Record<string, string>>({});

const missingTools = computed(() => (tools.value ?? []).filter((t) => !t.installed));

async function loadTools() {
  try {
    tools.value = await invoke<ToolStatus[]>("check_required_tools");
  } catch (e) {
    toolsError.value = String(e);
  }
}

// Per-tool busy flag, not one shared ref: installing `lm-sensors` must not
// disable the other rows' buttons -- they are independent packages.
async function installTool(tool: ToolStatus) {
  if (!tool.package || !nativeManager.value) return;
  installing.value = { ...installing.value, [tool.binary]: true };
  delete installErrors.value[tool.binary];
  try {
    await invoke<string>("install_package", { manager: nativeManager.value, package: tool.package });
    await loadTools();
  } catch (e) {
    installErrors.value = { ...installErrors.value, [tool.binary]: String(e) };
  } finally {
    installing.value = { ...installing.value, [tool.binary]: false };
  }
}

onMounted(async () => {
  try {
    results.value = await invoke<MissingDependency[]>("scan_missing_dependencies");
  } catch (e) {
    error.value = String(e);
  }
  try {
    nativeManager.value = await invoke<string | null>("detect_native_manager");
  } catch {
    nativeManager.value = null;
  }
  await loadTools();
});
</script>

<template>
  <div class="dep-page">
    <NxSectionHeader title="Dépendances" description="Outils système requis par NiTruX, et bibliothèques partagées des binaires courants." />

    <NxCard>
      <NxSectionHeader
        title="Outils système requis"
        :description="tools ? `${missingTools.length} manquant(s) sur ${tools.length}` : 'Vérification…'"
      />
      <NxCard v-if="toolsError" danger>{{ toolsError }}</NxCard>

      <div v-else-if="tools && missingTools.length === 0" class="dep-empty">
        Tous les outils utilisés par NiTruX sont installés.
      </div>

      <template v-else-if="tools">
        <div class="dep-note">
          Chaque outil absent désactive la fonctionnalité en face. NiTruX ne les installe pas d'office :
          installez uniquement ce dont vous avez besoin.
        </div>
        <div v-for="t in missingTools" :key="t.binary" class="dep-tool">
          <div class="dep-tool-main">
            <NxBadge status="warning">{{ t.binary }}</NxBadge>
            <span class="dep-feature">{{ t.feature }}</span>
          </div>
          <div class="dep-tool-action">
            <span v-if="t.package" class="dep-package">{{ t.package }}</span>
            <NxButton
              v-if="t.package && nativeManager"
              :disabled="installing[t.binary]"
              @click="installTool(t)"
            >
              {{ installing[t.binary] ? "Installation…" : "Installer" }}
            </NxButton>
          </div>
          <div v-if="installErrors[t.binary]" class="dep-tool-error">{{ installErrors[t.binary] }}</div>
        </div>
      </template>
    </NxCard>

    <NxSectionHeader title="Bibliothèques partagées" description="Vérifie qu'un ensemble de binaires système courants ont toutes leurs bibliothèques partagées résolues." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <div v-if="results && results.length === 0" class="dep-empty">Aucune dépendance manquante détectée.</div>

    <NxCard v-else-if="results">
      <div v-for="(r, i) in results" :key="`${r.binary}-${r.missing_library}-${i}`" class="dep-row">
        <span>{{ r.binary }}</span>
        <NxBadge status="danger">{{ r.missing_library }}</NxBadge>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.dep-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.dep-empty { color: var(--nx-text-secondary); }
.dep-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
.dep-note { color: var(--nx-text-secondary); font-size: 13px; margin-bottom: 8px; }
.dep-tool { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 10px; padding: 8px 0; border-bottom: 1px solid var(--nx-style-border-color); }
.dep-tool-main { display: flex; align-items: center; gap: 10px; min-width: 0; }
.dep-tool-action { display: flex; align-items: center; gap: 10px; }
.dep-feature { font-size: 13px; }
.dep-package { color: var(--nx-text-secondary); font-size: 12px; font-family: monospace; }
.dep-tool-error { flex-basis: 100%; color: var(--nx-danger, #e5484d); font-size: 12px; }
</style>
