<!-- src/pages/InstalledSoftwarePage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface InstalledPackage { name: string; version: string }

const packages = ref<InstalledPackage[] | null>(null);
const envVars = ref<[string, string][]>([]);
const softwareFilter = ref("");
const loadError = ref<string | null>(null);
const envVarsError = ref<string | null>(null);

// Kept in two independent try/catch blocks, not one shared try: these are
// unrelated data sources (installed-packages detection can genuinely fail
// with no native package manager found -- InstalledPackagesPage's own
// loadError below -- but that has nothing to do with reading the app's own
// environment variables). A shared try previously meant a package-listing
// failure silently left "Variables d'environnement" empty too. Both calls
// now get their own try/catch (get_environment_variables's Rust signature
// is bare/infallible-by-logic, but the IPC call itself can still reject at
// the Tauri layer -- same rare-but-real category guarded everywhere else
// in the app, see NetworkPage.vue cycle 387). Mirrors the already-
// established pattern in UninstallerPage.vue, which keeps
// list_installed_packages's own try/catch separate from the unrelated
// detect_native_manager call right after it.
onMounted(async () => {
  try {
    packages.value = await invoke<InstalledPackage[]>("list_installed_packages");
  } catch (e) {
    loadError.value = String(e);
  }
  try {
    envVars.value = await invoke<[string, string][]>("get_environment_variables");
  } catch (e) {
    envVarsError.value = String(e);
  }
});

const filteredPackages = computed(() =>
  (packages.value ?? []).filter((p) => p.name.toLowerCase().includes(softwareFilter.value.toLowerCase())),
);
</script>

<template>
  <div class="sw-page">
    <NxSectionHeader title="Logiciels installés" description="Applications installées manuellement (hors dépendances) et variables d'environnement." />

    <NxCard v-if="loadError" danger>{{ loadError }}</NxCard>

    <NxCard>
      <NxSectionHeader :title="`Paquets (${packages?.length ?? 0})`" />
      <NxInput v-model="softwareFilter" placeholder="Filtrer par nom..." aria-label="Filtrer les logiciels installés par nom" />
      <div v-for="p in filteredPackages" :key="p.name" class="sw-row">
        <span>{{ p.name }}</span><span>{{ p.version }}</span>
      </div>
      <div v-if="packages && filteredPackages.length === 0" class="sw-empty">Aucun paquet ne correspond à cette recherche.</div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Variables d'environnement" />
      <NxCard v-if="envVarsError" danger>{{ envVarsError }}</NxCard>
      <div v-for="[key, value] in envVars" :key="key" class="sw-row">
        <span>{{ key }}</span><span>{{ value }}</span>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.sw-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.sw-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 4px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
.sw-empty { color: var(--nx-text-secondary); font-size: 13px; }
</style>
