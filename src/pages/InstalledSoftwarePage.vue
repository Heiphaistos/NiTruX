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

onMounted(async () => {
  try {
    packages.value = await invoke<InstalledPackage[]>("list_installed_packages");
    envVars.value = await invoke<[string, string][]>("get_environment_variables");
  } catch (e) {
    loadError.value = String(e);
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
      <NxInput v-model="softwareFilter" placeholder="Filtrer par nom..." />
      <div v-for="p in filteredPackages" :key="p.name" class="sw-row">
        <span>{{ p.name }}</span><span>{{ p.version }}</span>
      </div>
      <div v-if="packages && filteredPackages.length === 0" class="sw-empty">Aucun paquet ne correspond à cette recherche.</div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Variables d'environnement" />
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
