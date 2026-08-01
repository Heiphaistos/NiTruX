<!-- src/pages/QuickInstallPage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import { appCatalog, type AppCatalogEntry } from "@/data/appCatalog";

type InstallState = "idle" | "installing" | "success" | "error";

const nativeManager = ref<string | null>(null);
const selectedCategory = ref<string>("Tous");
const installState = ref<Record<string, InstallState>>({});
const installErrors = ref<Record<string, string>>({});

// Detection kicks off on mount; `install()` awaits this same promise if a
// click lands before it resolves, so the install flow never races the
// detection call regardless of exactly when Vue flushes the resulting DOM
// update — the button itself is never gated on detection having finished.
let managerReady: Promise<string | null> | null = null;

onMounted(() => {
  managerReady = invoke<string | null>("detect_native_manager").then((result) => {
    nativeManager.value = result;
    return result;
  });
});

const categories = computed(() => ["Tous", ...new Set(appCatalog.map((e) => e.category))]);

const filteredCatalog = computed<AppCatalogEntry[]>(() =>
  selectedCategory.value === "Tous" ? appCatalog : appCatalog.filter((e) => e.category === selectedCategory.value),
);

function stateOf(entry: AppCatalogEntry): InstallState {
  return installState.value[entry.id] ?? "idle";
}

async function install(entry: AppCatalogEntry) {
  if (entry.installMethod !== "apt") return;
  const manager = nativeManager.value ?? (await managerReady);
  if (!manager) return;
  installState.value[entry.id] = "installing";
  delete installErrors.value[entry.id];
  try {
    await invoke<string>("install_package", { manager, package: entry.packageId });
    installState.value[entry.id] = "success";
  } catch (e) {
    installState.value[entry.id] = "error";
    installErrors.value[entry.id] = String(e);
  }
}
</script>

<template>
  <div class="qi-page">
    <NxSectionHeader
      title="Installation rapide"
      :description="nativeManager ? `Gestionnaire détecté : ${nativeManager}` : 'Détection du gestionnaire de paquets...'"
    />

    <div class="qi-chips">
      <button
        v-for="cat in categories"
        :key="cat"
        class="qi-chip"
        :class="{ active: selectedCategory === cat }"
        @click="selectedCategory = cat"
      >
        {{ cat }}
      </button>
    </div>

    <div class="qi-grid">
      <NxCard v-for="entry in filteredCatalog" :key="entry.id" class="qi-card">
        <div class="qi-card-header">
          <span class="qi-icon">{{ entry.icon }}</span>
          <div>
            <div class="qi-name">{{ entry.name }}</div>
            <div class="qi-desc">{{ entry.description }}</div>
          </div>
        </div>

        <template v-if="entry.installMethod !== 'apt'">
          <NxBadge status="info">Bientôt disponible ({{ entry.installMethod }})</NxBadge>
          <NxButton disabled>Installer</NxButton>
        </template>
        <template v-else-if="stateOf(entry) === 'success'">
          <NxBadge status="success">Installé</NxBadge>
        </template>
        <template v-else>
          <div v-if="stateOf(entry) === 'installing'" class="qi-progress"><div class="qi-progress-bar"></div></div>
          <NxCard v-if="stateOf(entry) === 'error'" danger class="qi-error">{{ installErrors[entry.id] }}</NxCard>
          <NxButton :disabled="stateOf(entry) === 'installing'" @click="install(entry)">
            {{ stateOf(entry) === "installing" ? "Installation..." : "Installer" }}
          </NxButton>
        </template>
      </NxCard>
    </div>
  </div>
</template>

<style scoped>
.qi-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.qi-chips { display: flex; gap: 8px; flex-wrap: wrap; }
.qi-chip { padding: 6px 14px; border-radius: 99px; border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-secondary); cursor: pointer; font: inherit; font-size: 12px; }
.qi-chip.active { color: var(--nx-text-primary); font-weight: 600; border-color: var(--nx-accent-primary); }
.qi-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 14px; }
.qi-card { display: flex; flex-direction: column; gap: 10px; }
.qi-card-header { display: flex; gap: 10px; align-items: flex-start; }
.qi-icon { font-size: 28px; line-height: 1; }
.qi-name { font-weight: 600; color: var(--nx-text-primary); }
.qi-desc { font-size: 12px; color: var(--nx-text-secondary); }
.qi-progress { width: 100%; height: 4px; border-radius: 2px; background: color-mix(in srgb, var(--nx-accent-primary) 15%, transparent); overflow: hidden; }
.qi-progress-bar { width: 40%; height: 100%; background: var(--nx-accent-primary); border-radius: 2px; animation: qi-slide 1.2s ease-in-out infinite; }
.qi-error { font-size: 12px; padding: 8px 10px; }
@keyframes qi-slide {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(350%); }
}
</style>
