<!-- src/pages/PackagesPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSelect from "@/components/ui/NxSelect.vue";
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

const MANAGER_OPTIONS = [
  { value: "apt", label: "apt" },
  { value: "dnf", label: "dnf" },
  { value: "pacman", label: "pacman" },
  { value: "zypper", label: "zypper" },
];

const installManager = ref<"apt" | "dnf" | "pacman" | "zypper">("apt");
const installPackageName = ref("");
const installResult = ref<string | null>(null);
const installError = ref<string | null>(null);
const installing = ref(false);

async function installOne() {
  installing.value = true;
  installError.value = null;
  installResult.value = null;
  try {
    installResult.value = await invoke<string>("install_package", {
      manager: installManager.value,
      package: installPackageName.value,
    });
  } catch (e) {
    installError.value = String(e);
  } finally {
    installing.value = false;
  }
}

const upgrading = ref(false);
const upgradeResult = ref<string | null>(null);
const upgradeError = ref<string | null>(null);

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
  <div class="pkg-page">
    <div class="pkg-header">
      <NxSectionHeader title="Gestionnaire de paquets" description="Installation directe et mises à jour via le gestionnaire de paquets du système." />
      <NxButton :disabled="loading" @click="refresh">
        {{ loading ? "Vérification..." : "Vérifier les mises à jour" }}
      </NxButton>
    </div>

    <NxCard class="pkg-install-card">
      <div class="pkg-install-row">
        <NxSelect v-model="installManager" :options="MANAGER_OPTIONS" aria-label="Gestionnaire de paquets" />
        <NxInput v-model="installPackageName" placeholder="Nom du paquet à installer..." aria-label="Nom du paquet à installer" />
        <NxButton :disabled="installing" @click="installOne">{{ installing ? "Installation..." : "Installer" }}</NxButton>
        <NxButton :disabled="upgrading || updates.length === 0" @click="upgradeAll">{{ upgrading ? "Mise à jour..." : "Tout mettre à jour" }}</NxButton>
      </div>
      <NxCard v-if="installError" danger>{{ installError }}</NxCard>
      <NxBadge v-if="installResult" status="success">Installation terminée.</NxBadge>
      <NxCard v-if="upgradeError" danger>{{ upgradeError }}</NxCard>
      <NxBadge v-if="upgradeResult" status="success">Mise à jour terminée.</NxBadge>
    </NxCard>

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <div v-else-if="!loading && updates.length === 0" class="pkg-empty">
      Aucune mise à jour disponible.
    </div>

    <NxCard v-else>
      <div class="pkg-table-scroll">
        <table class="pkg-table">
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
.pkg-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.pkg-header { display: flex; justify-content: space-between; align-items: flex-start; gap: 12px; flex-wrap: wrap; }
.pkg-install-card { display: flex; flex-direction: column; gap: 10px; }
.pkg-install-row { display: flex; gap: 10px; align-items: center; }
.pkg-empty { color: var(--nx-text-secondary); }
.pkg-table-scroll { overflow-x: auto; }
.pkg-table { width: 100%; min-width: 480px; border-collapse: collapse; font-size: 13px; }
.pkg-table th { text-align: left; color: var(--nx-text-secondary); border-bottom: 1px solid var(--nx-style-border-color); padding: 8px; }
.pkg-table td { padding: 8px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
