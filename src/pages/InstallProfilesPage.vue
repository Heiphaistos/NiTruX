<!-- src/pages/InstallProfilesPage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import { appCatalog, type AppCatalogEntry } from "@/data/appCatalog";
import { installProfiles } from "@/data/installProfiles";

const nativeManager = ref<string | null>(null);
let managerReady: Promise<string | null> | null = null;
onMounted(() => {
  managerReady = invoke<string | null>("detect_native_manager").then((result) => {
    nativeManager.value = result;
    return result;
  });
});

const catalogById = new Map<string, AppCatalogEntry>(appCatalog.map((e) => [e.id, e]));

const selected = ref<Set<string>>(new Set());

function toggle(id: string) {
  const next = new Set(selected.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  selected.value = next;
}

function selectProfile(appIds: string[]) {
  selected.value = new Set(appIds);
}

const selectedEntries = computed<AppCatalogEntry[]>(() =>
  [...selected.value].map((id) => catalogById.get(id)).filter((e): e is AppCatalogEntry => !!e),
);

interface InstallResult { id: string; name: string; success: boolean; message: string }

const installing = ref(false);
const results = ref<InstallResult[] | null>(null);

async function installOne(entry: AppCatalogEntry): Promise<InstallResult> {
  try {
    if (entry.installMethod === "apt") {
      const manager = nativeManager.value ?? (await managerReady);
      if (!manager) throw new Error("aucun gestionnaire de paquets natif détecté");
      const message = await invoke<string>("install_package", { manager, package: entry.packageId });
      return { id: entry.id, name: entry.name, success: true, message };
    }
    if (entry.installMethod === "flatpak") {
      const message = await invoke<string>("install_flatpak_package", { appId: entry.packageId });
      return { id: entry.id, name: entry.name, success: true, message };
    }
    const message = await invoke<string>("install_snap_package", { package: entry.packageId });
    return { id: entry.id, name: entry.name, success: true, message };
  } catch (e) {
    return { id: entry.id, name: entry.name, success: false, message: String(e) };
  }
}

async function installSelection() {
  installing.value = true;
  results.value = [];
  // Sequential, not parallel -- avoids concurrent lock contention on the
  // native package manager (apt/dnf/etc hold an exclusive lock; parallel
  // installs would just serialize at the OS level anyway, but with
  // confusing partial-failure errors instead of a clean queue).
  for (const entry of selectedEntries.value) {
    const result = await installOne(entry);
    results.value = [...results.value, result];
  }
  installing.value = false;
}
</script>

<template>
  <div class="ip-page">
    <NxSectionHeader
      title="Installation par profils"
      description="Sélectionnez un profil ou des applications individuelles, puis installez tout en une fois."
    />

    <div class="ip-profiles">
      <button
        v-for="profile in installProfiles"
        :key="profile.id"
        class="ip-profile-button"
        @click="selectProfile(profile.appIds)"
      >
        <strong>{{ profile.label }}</strong>
        <p>{{ profile.description }}</p>
        <span class="ip-profile-count">{{ profile.appIds.length }} applications</span>
      </button>
    </div>

    <NxCard>
      <NxSectionHeader title="Sélection" />
      <label v-for="entry in appCatalog" :key="entry.id" class="ip-check-row">
        <input type="checkbox" :checked="selected.has(entry.id)" @change="toggle(entry.id)" />
        <span>{{ entry.icon }} {{ entry.name }}</span>
      </label>
      <NxButton :disabled="selected.size === 0 || installing" @click="installSelection">
        {{ installing ? "Installation..." : "Installer la sélection" }}
      </NxButton>
    </NxCard>

    <NxCard v-if="results && results.length > 0">
      <NxSectionHeader title="Résultat" />
      <div v-for="r in results" :key="r.id" class="ip-result-row">
        <span>{{ r.name }}</span>
        <NxBadge :status="r.success ? 'success' : 'danger'">{{ r.success ? 'installé' : 'échec' }}</NxBadge>
        <span v-if="!r.success" class="ip-result-message">{{ r.message }}</span>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.ip-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.ip-profiles { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 12px; }
.ip-profile-button { text-align: left; padding: 14px; border-radius: 10px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); cursor: pointer; font: inherit; }
.ip-profile-button p { margin: 6px 0; font-size: 12px; color: var(--nx-text-secondary); }
.ip-profile-count { font-size: 11px; color: var(--nx-text-secondary); }
.ip-check-row { display: flex; align-items: center; gap: 8px; padding: 4px 0; font-size: 13px; }
.ip-result-row { display: flex; align-items: center; gap: 10px; padding: 4px 0; font-size: 13px; }
.ip-result-message { color: var(--nx-text-secondary); font-size: 12px; }
</style>
