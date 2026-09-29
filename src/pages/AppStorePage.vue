<!-- src/pages/AppStorePage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import {
  installCandidate,
  sourceLabel,
  type InstallSources,
  type InstallSource,
  type SearchOutcome,
  type SearchResult,
} from "@/lib/appInstall";

// Live search across every installation source this machine has (native
// repositories, Flathub, Snap Store): anything installable, not only the
// curated catalog of "Installation rapide".
const sources = ref<InstallSources | null>(null);
const sourcesError = ref<string | null>(null);
const query = ref("");
const searching = ref(false);
const outcome = ref<SearchOutcome | null>(null);
const searchError = ref<string | null>(null);
const sourceFilter = ref<"all" | InstallSource>("all");

type State = { status: "installing" | "ok" | "error"; message: string };
const states = ref<Record<string, State>>({});
const setupBusy = ref<"flatpak" | "snap" | null>(null);
// Native packages already installed, to badge search results.
const installedNative = ref<Set<string>>(new Set());
const setupMessage = ref<{ ok: boolean; text: string } | null>(null);

const keyOf = (r: SearchResult) => `${r.source}:${r.manager ?? ""}:${r.id}`;

async function loadInstalled() {
  try {
    const pkgs = await invoke<{ name: string }[]>("list_installed_packages");
    installedNative.value = new Set(pkgs.map((p) => p.name));
  } catch {
    installedNative.value = new Set();
  }
}

function alreadyInstalled(r: SearchResult): boolean {
  return r.source === "native" && installedNative.value.has(r.id);
}

async function loadSources() {
  sourcesError.value = null;
  try {
    sources.value = await invoke<InstallSources>("get_install_sources");
  } catch (e) {
    sourcesError.value = String(e);
  }
}

async function runSearch() {
  if (!query.value.trim()) return;
  searching.value = true;
  searchError.value = null;
  try {
    outcome.value = await invoke<SearchOutcome>("search_packages", { query: query.value });
  } catch (e) {
    searchError.value = String(e);
    outcome.value = null;
  } finally {
    searching.value = false;
  }
}

async function install(r: SearchResult) {
  const k = keyOf(r);
  states.value[k] = { status: "installing", message: "" };
  try {
    const output = await installCandidate({ source: r.source, id: r.id, manager: r.manager });
    states.value[k] = { status: "ok", message: output.trim() || "Installation terminée." };
  } catch (e) {
    states.value[k] = { status: "error", message: String(e) };
  }
}

async function setup(kind: "flatpak" | "snap") {
  setupBusy.value = kind;
  setupMessage.value = null;
  try {
    const out = await invoke<string>(kind === "flatpak" ? "setup_flatpak" : "setup_snap");
    setupMessage.value = { ok: true, text: out.trim() || "Source activée." };
    await loadSources();
  } catch (e) {
    setupMessage.value = { ok: false, text: String(e) };
  } finally {
    setupBusy.value = null;
  }
}

const filtered = computed(() =>
  (outcome.value?.results ?? []).filter((r) => sourceFilter.value === "all" || r.source === sourceFilter.value),
);

const counts = computed(() => {
  const c: Record<string, number> = { all: 0, native: 0, flatpak: 0, snap: 0 };
  for (const r of outcome.value?.results ?? []) {
    c.all++;
    c[r.source]++;
  }
  return c;
});

onMounted(() => { loadSources(); loadInstalled(); });
</script>

<template>
  <div class="st-page">
    <NxSectionHeader
      title="Magasin d'applications"
      description="Recherche en direct dans tous les dépôts de la machine (natif, Flathub, Snap) et installation en un clic."
    />

    <NxCard class="st-sources">
      <NxCard v-if="sourcesError" danger>{{ sourcesError }}</NxCard>
      <template v-else-if="sources">
        <div class="st-source">
          <NxBadge :status="sources.native ? 'success' : 'warning'">Natif</NxBadge>
          <span>{{ sources.native ?? "aucun gestionnaire détecté" }}</span>
        </div>
        <div class="st-source">
          <NxBadge :status="sources.flatpak && sources.flathub ? 'success' : 'warning'">Flatpak</NxBadge>
          <span v-if="sources.flatpak && sources.flathub">Flathub prêt</span>
          <template v-else>
            <span>{{ sources.flatpak ? "Flathub non configuré" : "non installé" }}</span>
            <NxButton :disabled="setupBusy !== null" @click="setup('flatpak')">
              {{ setupBusy === "flatpak" ? "Activation..." : "Activer Flatpak + Flathub" }}
            </NxButton>
          </template>
        </div>
        <div class="st-source">
          <NxBadge :status="sources.snap && sources.snapd_running ? 'success' : 'warning'">Snap</NxBadge>
          <span v-if="sources.snap && sources.snapd_running">Snap Store prêt</span>
          <template v-else>
            <span>{{ sources.snap ? "service snapd arrêté" : "non installé" }}</span>
            <NxButton :disabled="setupBusy !== null" @click="setup('snap')">
              {{ setupBusy === "snap" ? "Activation..." : "Activer Snap" }}
            </NxButton>
          </template>
        </div>
      </template>
      <p v-else class="st-muted">Détection des sources…</p>
      <NxBadge v-if="setupMessage?.ok" status="success" live>{{ setupMessage.text }}</NxBadge>
      <NxCard v-if="setupMessage && !setupMessage.ok" danger class="st-pre">{{ setupMessage.text }}</NxCard>
    </NxCard>

    <form class="st-search" @submit.prevent="runSearch">
      <NxInput v-model="query" placeholder="Nom d'une application (ex : vlc, gimp, discord)..." aria-label="Rechercher une application" />
      <NxButton :disabled="searching || !query.trim()" @click="runSearch">{{ searching ? "Recherche..." : "Rechercher" }}</NxButton>
    </form>

    <NxCard v-if="searchError" danger>{{ searchError }}</NxCard>
    <NxCard v-for="e in outcome?.errors ?? []" :key="e" danger>{{ e }}</NxCard>

    <template v-if="outcome">
      <div class="st-filters">
        <button
          v-for="f in (['all', 'native', 'flatpak', 'snap'] as const)"
          :key="f"
          class="st-chip"
          :class="{ active: sourceFilter === f }"
          @click="sourceFilter = f"
        >
          {{ f === "all" ? "Tout" : f === "native" ? sources?.native ?? "Natif" : f === "flatpak" ? "Flatpak" : "Snap" }}
          ({{ counts[f] }})
        </button>
      </div>
      <p v-if="filtered.length === 0" class="st-muted">Aucun résultat.</p>
      <NxCard v-for="r in filtered" :key="keyOf(r)" class="st-result">
        <div class="st-result-main">
          <div class="st-result-title">
            <strong>{{ r.name }}</strong>
            <NxBadge status="info">{{ sourceLabel(r) }}</NxBadge>
            <span v-if="r.version" class="st-muted">{{ r.version }}</span>
          </div>
          <div class="st-muted">{{ r.summary }}</div>
          <code v-if="r.id !== r.name" class="st-id">{{ r.id }}</code>
        </div>
        <div class="st-result-action">
          <NxBadge v-if="states[keyOf(r)]?.status === 'ok'" status="success" live>Installé ✓</NxBadge>
          <NxBadge v-else-if="alreadyInstalled(r)" status="success">Déjà installé</NxBadge>
          <NxButton v-else :disabled="states[keyOf(r)]?.status === 'installing'" @click="install(r)">
            {{ states[keyOf(r)]?.status === "installing" ? "Installation..." : "Installer" }}
          </NxButton>
        </div>
        <NxCard v-if="states[keyOf(r)]?.status === 'error'" danger class="st-pre st-full">{{ states[keyOf(r)].message }}</NxCard>
      </NxCard>
    </template>
  </div>
</template>

<style scoped>
.st-page { padding: 24px; display: flex; flex-direction: column; gap: 14px; }
.st-sources { display: flex; flex-direction: column; gap: 8px; }
.st-source { display: flex; align-items: center; gap: 10px; font-size: 13px; flex-wrap: wrap; }
.st-search { display: flex; gap: 10px; align-items: center; }
.st-filters { display: flex; gap: 8px; flex-wrap: wrap; }
.st-chip { padding: 6px 14px; border-radius: 99px; border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-secondary); cursor: pointer; font: inherit; font-size: 12px; }
.st-chip.active { color: var(--nx-text-primary); font-weight: 600; border-color: var(--nx-accent-primary); }
.st-result { display: flex; flex-wrap: wrap; align-items: center; gap: 12px; }
.st-result-main { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
.st-result-title { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.st-result-action { flex-shrink: 0; }
.st-full { flex-basis: 100%; }
.st-muted { font-size: 12px; color: var(--nx-text-secondary); margin: 0; }
.st-id { font-size: 11px; color: var(--nx-text-secondary); }
.st-pre { white-space: pre-wrap; font-size: 12px; }
</style>
