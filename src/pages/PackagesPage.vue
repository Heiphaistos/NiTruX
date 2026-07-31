<!-- src/pages/PackagesPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

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

function sourceBadgeClass(source: string): string {
  return `pkg-badge pkg-badge--${source}`;
}

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
      <h1>Paquets & mises à jour</h1>
      <button class="pkg-refresh" :disabled="loading" @click="refresh">
        {{ loading ? "Vérification..." : "Vérifier les mises à jour" }}
      </button>
    </div>

    <div class="pkg-install-row">
      <select v-model="installManager">
        <option value="apt">apt</option>
        <option value="dnf">dnf</option>
        <option value="pacman">pacman</option>
        <option value="zypper">zypper</option>
      </select>
      <input v-model="installPackageName" class="pkg-input" placeholder="Nom du paquet à installer..." />
      <button :disabled="installing" @click="installOne">{{ installing ? "Installation..." : "Installer" }}</button>
      <button :disabled="upgrading" @click="upgradeAll">{{ upgrading ? "Mise à jour..." : "Tout mettre à jour" }}</button>
    </div>
    <div v-if="installError" class="pkg-error">{{ installError }}</div>
    <div v-if="installResult" class="pkg-success">Installation terminée.</div>
    <div v-if="upgradeError" class="pkg-error">{{ upgradeError }}</div>
    <div v-if="upgradeResult" class="pkg-success">Mise à jour terminée.</div>

    <div v-if="error" class="pkg-error">{{ error }}</div>

    <div v-else-if="!loading && updates.length === 0" class="pkg-empty">
      Aucune mise à jour disponible.
    </div>

    <table v-else class="pkg-table">
      <thead>
        <tr><th>Source</th><th>Paquet</th><th>Version actuelle</th><th>Nouvelle version</th></tr>
      </thead>
      <tbody>
        <tr v-for="u in updates" :key="`${u.source}-${u.name}`">
          <td><span :class="sourceBadgeClass(u.source)">{{ u.source }}</span></td>
          <td>{{ u.name }}</td>
          <td>{{ u.current_version || "—" }}</td>
          <td>{{ u.new_version }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.pkg-page { padding: 24px; color: var(--nx-text-primary); }
.pkg-header { display: flex; justify-content: space-between; align-items: center; }
.pkg-refresh { padding: 8px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); cursor: pointer; }
.pkg-refresh:disabled { opacity: 0.6; cursor: default; }
.pkg-error { margin-top: 16px; padding: 10px 14px; border-radius: 8px; background: color-mix(in srgb, var(--nx-accent-danger) 15%, transparent); border: 1px solid var(--nx-accent-danger); }
.pkg-empty { margin-top: 16px; color: var(--nx-text-secondary); }
.pkg-install-row { display: flex; gap: 10px; align-items: center; margin-bottom: 12px; }
.pkg-success { padding: 10px 14px; border-radius: 8px; background: color-mix(in srgb, var(--nx-accent-success) 15%, transparent); border: 1px solid var(--nx-accent-success); margin-bottom: 10px; }
.pkg-table { width: 100%; border-collapse: collapse; margin-top: 16px; font-size: 13px; }
.pkg-table th { text-align: left; color: var(--nx-text-secondary); border-bottom: 1px solid var(--nx-border); padding: 8px; }
.pkg-table td { padding: 8px; border-bottom: 1px solid var(--nx-border); }
.pkg-badge { display: inline-block; padding: 2px 8px; border-radius: 999px; font-size: 11px; background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); }
</style>
