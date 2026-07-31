<!-- src/pages/SecurityPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface FirewallStatus { active: boolean; rules: string[] }
interface MalwareFinding { path: string; signature: string }
interface SnapshotInfo { id: string; date: string }

type Tab = "firewall" | "malware" | "snapshots";
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
</script>

<template>
  <div class="sec-page">
    <h1>Sécurité & maintenance</h1>

    <div class="sec-tabs">
      <button :class="{ active: activeTab === 'firewall' }" @click="onTabClick('firewall')">Pare-feu</button>
      <button :class="{ active: activeTab === 'malware' }" @click="onTabClick('malware')">Scan malware</button>
      <button :class="{ active: activeTab === 'snapshots' }" @click="onTabClick('snapshots')">Snapshots</button>
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
      <div v-for="f in findings" :key="f.path" class="sec-row sec-finding">
        <span>{{ f.path }}</span>
        <span>{{ f.signature }}</span>
      </div>
    </section>

    <section v-else-if="activeTab === 'snapshots'" class="sec-panel">
      <div v-if="snapshotsError" class="sec-error">{{ snapshotsError }}</div>
      <div v-for="s in snapshots" :key="s.id" class="sec-row">
        <span>#{{ s.id }}</span>
        <span>{{ s.date }}</span>
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
</style>
