<!-- src/pages/SecurityPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface FirewallStatus { active: boolean; rules: string[] }
interface MalwareFinding { path: string; signature: string }
interface SnapshotInfo { id: string; date: string }

type Tab = "firewall" | "malware" | "snapshots" | "troubleshoot";
const activeTab = ref<Tab>("firewall");

const firewall = ref<FirewallStatus | null>(null);
const firewallError = ref<string | null>(null);

async function loadFirewall() {
  firewallError.value = null;
  try {
    firewall.value = await invoke<FirewallStatus>("get_firewall_status");
  } catch (e) {
    firewallError.value = String(e);
  }
}
onMounted(loadFirewall);

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
const troubleshootBusy = ref<string | null>(null); // holds the action id currently running, or null
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

const quarantining = ref<string | null>(null); // holds the path currently being quarantined, or null
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
  <div class="sec-page">
    <h1>Sécurité & maintenance</h1>

    <div class="sec-tabs">
      <button :class="{ active: activeTab === 'firewall' }" @click="onTabClick('firewall')">Pare-feu</button>
      <button :class="{ active: activeTab === 'malware' }" @click="onTabClick('malware')">Scan malware</button>
      <button :class="{ active: activeTab === 'snapshots' }" @click="onTabClick('snapshots')">Snapshots</button>
      <button :class="{ active: activeTab === 'troubleshoot' }" @click="onTabClick('troubleshoot')">Dépannage</button>
    </div>

    <section v-if="activeTab === 'firewall'" class="sec-panel">
      <div v-if="firewallError" class="sec-error">{{ firewallError }}</div>
      <template v-else-if="firewall">
        <div class="sec-status" :class="firewall.active ? 'sec-active' : 'sec-inactive'">
          UFW {{ firewall.active ? "actif" : "inactif" }}
        </div>
        <div v-for="(r, i) in firewall.rules" :key="i" class="sec-row">{{ r }}</div>
      </template>
    </section>

    <section v-else-if="activeTab === 'malware'" class="sec-panel">
      <div class="sec-form-row">
        <input v-model="scanDir" class="sec-input" placeholder="Dossier à scanner..." />
        <button :disabled="scanning" @click="runScan">{{ scanning ? "Scan en cours..." : "Scanner" }}</button>
      </div>
      <div v-if="scanError" class="sec-error">{{ scanError }}</div>
      <div v-else-if="scanDone && findings.length === 0" class="sec-empty">Aucune menace détectée.</div>
      <div v-if="quarantineError" class="sec-error">{{ quarantineError }}</div>
      <div v-for="f in findings" :key="f.path" class="sec-row sec-finding">
        <span>{{ f.path }}</span>
        <span>{{ f.signature }}</span>
        <button :disabled="quarantining !== null" @click="quarantineFinding(f.path)">
          {{ quarantining === f.path ? "Mise en quarantaine..." : "Mettre en quarantaine" }}
        </button>
      </div>
    </section>

    <section v-else-if="activeTab === 'snapshots'" class="sec-panel">
      <div class="sec-form-row">
        <button :disabled="snapshotCreating" @click="createSnapshotNow">{{ snapshotCreating ? "Création..." : "Créer un instantané" }}</button>
      </div>
      <div v-if="snapshotCreateError" class="sec-error">{{ snapshotCreateError }}</div>
      <div v-if="snapshotsError" class="sec-error">{{ snapshotsError }}</div>
      <div v-for="s in snapshots" :key="s.id" class="sec-row">
        <span>#{{ s.id }}</span>
        <span>{{ s.date }}</span>
      </div>
    </section>

    <section v-else-if="activeTab === 'troubleshoot'" class="sec-panel">
      <div v-if="troubleshootError" class="sec-error">{{ troubleshootError }}</div>
      <div v-if="troubleshootResult" class="sec-success">{{ troubleshootResult }}</div>
      <div v-for="a in TROUBLESHOOT_ACTIONS" :key="a.id" class="sec-form-row">
        <span class="sec-troubleshoot-label">{{ a.label }}</span>
        <button :disabled="troubleshootBusy !== null" @click="runTroubleshootAction(a.id)">
          {{ troubleshootBusy === a.id ? "En cours..." : "Exécuter" }}
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.sec-page { padding: 24px; color: var(--nx-text-primary); }
.sec-tabs { display: flex; gap: 8px; margin: 16px 0; }
.sec-tabs button { padding: 8px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-secondary); cursor: pointer; }
.sec-tabs button.active { color: var(--nx-text-primary); border-color: var(--nx-accent-primary); }
.sec-error { padding: 10px 14px; border-radius: 8px; background: color-mix(in srgb, var(--nx-accent-danger) 15%, transparent); border: 1px solid var(--nx-accent-danger); }
.sec-status { padding: 10px 14px; border-radius: 8px; margin-bottom: 10px; font-weight: 600; }
.sec-active { background: color-mix(in srgb, var(--nx-accent-success) 15%, transparent); border: 1px solid var(--nx-accent-success); }
.sec-inactive { background: color-mix(in srgb, var(--nx-accent-warning) 15%, transparent); border: 1px solid var(--nx-accent-warning); }
.sec-row { display: flex; justify-content: space-between; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-border); }
.sec-finding { color: var(--nx-accent-danger); }
.sec-form-row { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; }
.sec-input { flex: 1; padding: 8px 10px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); }
.sec-empty { color: var(--nx-text-secondary); margin-top: 10px; }
.sec-success { padding: 10px 14px; border-radius: 8px; background: color-mix(in srgb, var(--nx-accent-success) 15%, transparent); border: 1px solid var(--nx-accent-success); margin-bottom: 10px; }
.sec-troubleshoot-label { flex: 1; }
</style>
