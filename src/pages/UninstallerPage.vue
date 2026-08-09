<!-- src/pages/UninstallerPage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface InstalledPackage { name: string; version: string }

const packages = ref<InstalledPackage[]>([]);
const loadError = ref<string | null>(null);
const nativeManager = ref<string | null>(null);
const managerError = ref<string | null>(null);
const searchText = ref("");

onMounted(async () => {
  try {
    packages.value = await invoke<InstalledPackage[]>("list_installed_packages");
  } catch (e) {
    loadError.value = String(e);
  }
  try {
    nativeManager.value = await invoke<string | null>("detect_native_manager");
  } catch (e) {
    // Without this, a rejection here left the header stuck on "Détection
    // du gestionnaire..." forever (implying detection is still running)
    // instead of showing that it actually failed.
    managerError.value = String(e);
  }
});

const filteredPackages = computed(() =>
  searchText.value === ""
    ? packages.value
    : packages.value.filter((p) => p.name.toLowerCase().includes(searchText.value.toLowerCase())),
);

const confirmingPackage = ref<string | null>(null);
const confirmText = ref("");
const uninstalling = ref<string | null>(null);
const uninstallError = ref<string | null>(null);
const uninstallResult = ref<string | null>(null);

function startConfirm(name: string) {
  confirmingPackage.value = name;
  confirmText.value = "";
  uninstallError.value = null;
  uninstallResult.value = null;
}

async function confirmUninstall(name: string) {
  if (!nativeManager.value) {
    uninstallError.value = "aucun gestionnaire de paquets natif détecté";
    return;
  }
  uninstalling.value = name;
  uninstallError.value = null;
  try {
    uninstallResult.value = await invoke<string>("uninstall_package", { manager: nativeManager.value, package: name });
    packages.value = packages.value.filter((p) => p.name !== name);
    confirmingPackage.value = null;
  } catch (e) {
    uninstallError.value = String(e);
  } finally {
    uninstalling.value = null;
  }
}
</script>

<template>
  <div class="unin-page">
    <NxSectionHeader title="Désinstalleur" :description="nativeManager ? `Gestionnaire détecté : ${nativeManager}` : 'Détection du gestionnaire...'" />

    <NxCard v-if="loadError" danger>{{ loadError }}</NxCard>
    <NxCard v-if="managerError" danger>{{ managerError }}</NxCard>
    <NxCard v-if="uninstallError" danger>{{ uninstallError }}</NxCard>
    <NxBadge v-if="uninstallResult" status="success" live>{{ uninstallResult }}</NxBadge>

    <NxInput v-model="searchText" placeholder="Rechercher un paquet..." aria-label="Rechercher un paquet" />

    <NxCard v-for="pkg in filteredPackages" :key="pkg.name" class="unin-card">
      <div class="unin-row">
        <span>{{ pkg.name }}</span>
        <span class="unin-version">{{ pkg.version || "—" }}</span>
        <NxButton v-if="confirmingPackage !== pkg.name" variant="danger" @click="startConfirm(pkg.name)">Désinstaller</NxButton>
      </div>
      <div v-if="confirmingPackage === pkg.name" class="unin-confirm-row">
        <NxInput v-model="confirmText" :placeholder="`Tapez « ${pkg.name} » pour confirmer`" aria-label="Confirmation du nom du paquet à désinstaller" />
        <NxButton
          variant="danger"
          :disabled="uninstalling !== null || confirmText !== pkg.name"
          @click="confirmUninstall(pkg.name)"
        >
          {{ uninstalling === pkg.name ? "Désinstallation..." : "Confirmer la désinstallation" }}
        </NxButton>
      </div>
    </NxCard>
    <NxCard v-if="!loadError && filteredPackages.length === 0" class="unin-empty">
      {{ packages.length === 0 ? "Aucun paquet installé trouvé." : "Aucun paquet ne correspond à cette recherche." }}
    </NxCard>
  </div>
</template>

<style scoped>
.unin-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.unin-card { display: flex; flex-direction: column; gap: 10px; }
.unin-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; }
.unin-version { color: var(--nx-text-secondary); font-size: 12px; }
.unin-confirm-row { display: flex; gap: 10px; align-items: center; }
.unin-empty { color: var(--nx-text-secondary); font-size: 13px; }
</style>
