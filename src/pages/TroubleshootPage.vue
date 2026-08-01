<!-- src/pages/TroubleshootPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface MalwareFinding { path: string; signature: string }
interface SnapshotInfo { id: string; date: string }

type Tab = "malware" | "snapshots" | "troubleshoot";
const activeTab = ref<Tab>("malware");

const scanDir = ref("");
const findings = ref<MalwareFinding[]>([]);
const scanError = ref<string | null>(null);
const scanning = ref(false);
const scanDone = ref(false);

async function runScan() {
  scanning.value = true;
  scanError.value = null;
  scanDone.value = false;
  try {
    findings.value = await invoke<MalwareFinding[]>("scan_for_malware", { directory: scanDir.value });
    scanDone.value = true;
  } catch (e) {
    scanError.value = String(e);
  } finally {
    scanning.value = false;
  }
}

const snapshots = ref<SnapshotInfo[]>([]);
const snapshotsError = ref<string | null>(null);

async function loadSnapshots() {
  snapshotsError.value = null;
  try {
    snapshots.value = await invoke<SnapshotInfo[]>("list_snapshots");
  } catch (e) {
    snapshotsError.value = String(e);
  }
}

function onTabClick(tab: Tab) {
  activeTab.value = tab;
  if (tab === "snapshots" && snapshots.value.length === 0 && !snapshotsError.value) {
    loadSnapshots();
  }
}

const TROUBLESHOOT_ACTIONS: { id: string; label: string }[] = [
  { id: "clean-cache", label: "Vider le cache des paquets" },
  { id: "fix-broken", label: "Réparer les paquets cassés" },
  { id: "restart-network", label: "Redémarrer le réseau" },
  { id: "vacuum-logs", label: "Purger les anciens journaux" },
];
const troubleshootBusy = ref<string | null>(null);
const troubleshootResult = ref<string | null>(null);
const troubleshootError = ref<string | null>(null);

async function runTroubleshootAction(actionId: string) {
  troubleshootBusy.value = actionId;
  troubleshootError.value = null;
  troubleshootResult.value = null;
  try {
    troubleshootResult.value = await invoke<string>("run_troubleshoot_action", { action: actionId });
  } catch (e) {
    troubleshootError.value = String(e);
  } finally {
    troubleshootBusy.value = null;
  }
}

const snapshotCreating = ref(false);
const snapshotCreateError = ref<string | null>(null);

async function createSnapshotNow() {
  snapshotCreating.value = true;
  snapshotCreateError.value = null;
  try {
    await invoke("create_snapshot");
    await loadSnapshots();
  } catch (e) {
    snapshotCreateError.value = String(e);
  } finally {
    snapshotCreating.value = false;
  }
}

const quarantining = ref<string | null>(null);
const quarantineError = ref<string | null>(null);

async function quarantineFinding(path: string) {
  quarantining.value = path;
  quarantineError.value = null;
  try {
    await invoke("quarantine_file", { path });
    findings.value = findings.value.filter((f) => f.path !== path);
  } catch (e) {
    quarantineError.value = String(e);
  } finally {
    quarantining.value = null;
  }
}
</script>

<template>
  <div class="ts-page">
    <NxSectionHeader title="Dépannage" description="Analyse antivirus, instantanés système et actions de maintenance." />

    <div class="ts-tabs">
      <button :class="{ active: activeTab === 'malware' }" @click="onTabClick('malware')">Scan malware</button>
      <button :class="{ active: activeTab === 'snapshots' }" @click="onTabClick('snapshots')">Snapshots</button>
      <button :class="{ active: activeTab === 'troubleshoot' }" @click="onTabClick('troubleshoot')">Dépannage</button>
    </div>

    <NxCard v-if="activeTab === 'malware'">
      <div class="ts-form-row">
        <NxInput v-model="scanDir" placeholder="Dossier à scanner..." />
        <NxButton :disabled="scanning" @click="runScan">{{ scanning ? "Scan en cours..." : "Scanner" }}</NxButton>
      </div>
      <NxCard v-if="scanError" danger>{{ scanError }}</NxCard>
      <div v-else-if="scanDone && findings.length === 0" class="ts-empty">Aucune menace détectée.</div>
      <NxCard v-if="quarantineError" danger>{{ quarantineError }}</NxCard>
      <div v-for="f in findings" :key="f.path" class="ts-finding-row">
        <span>{{ f.path }}</span>
        <span>{{ f.signature }}</span>
        <NxButton variant="danger" :disabled="quarantining !== null" @click="quarantineFinding(f.path)">
          {{ quarantining === f.path ? "Mise en quarantaine..." : "Mettre en quarantaine" }}
        </NxButton>
      </div>
    </NxCard>

    <NxCard v-else-if="activeTab === 'snapshots'">
      <div class="ts-form-row">
        <NxButton :disabled="snapshotCreating" @click="createSnapshotNow">{{ snapshotCreating ? "Création..." : "Créer un instantané" }}</NxButton>
      </div>
      <NxCard v-if="snapshotCreateError" danger>{{ snapshotCreateError }}</NxCard>
      <NxCard v-if="snapshotsError" danger>{{ snapshotsError }}</NxCard>
      <div v-for="s in snapshots" :key="s.id" class="ts-row">
        <span>#{{ s.id }}</span>
        <span>{{ s.date }}</span>
      </div>
    </NxCard>

    <NxCard v-else>
      <NxCard v-if="troubleshootError" danger>{{ troubleshootError }}</NxCard>
      <NxBadge v-if="troubleshootResult" status="success">{{ troubleshootResult }}</NxBadge>
      <div v-for="a in TROUBLESHOOT_ACTIONS" :key="a.id" class="ts-form-row">
        <span class="ts-action-label">{{ a.label }}</span>
        <NxButton :disabled="troubleshootBusy !== null" @click="runTroubleshootAction(a.id)">
          {{ troubleshootBusy === a.id ? "En cours..." : "Exécuter" }}
        </NxButton>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.ts-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.ts-tabs { display: flex; gap: 8px; }
.ts-tabs button { padding: 8px 14px; border-radius: var(--nx-style-radius); border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-secondary); cursor: pointer; font: inherit; }
.ts-tabs button.active { color: var(--nx-text-primary); font-weight: 600; }
.ts-form-row { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; }
.ts-empty { color: var(--nx-text-secondary); }
.ts-finding-row, .ts-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
.ts-action-label { flex: 1; }
</style>
