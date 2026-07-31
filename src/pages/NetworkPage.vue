<!-- src/pages/NetworkPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface WifiNetwork { ssid: string; security: string; signal_percent: number; connected: boolean }
interface ListeningPort { port: number; process: string | null }
interface NetworkSnapshot { wifi_networks: WifiNetwork[]; listening_ports: ListeningPort[]; dns_servers: string[]; hosts_file: string }
interface PortResult { port: number; open: boolean }
interface Container { id: string; image: string; name: string; status: string }
interface DockerImageInfo { id: string; repository: string; tag: string; size: string }
interface DockerSnapshot { available: boolean; containers: Container[]; images: DockerImageInfo[] }

type Tab = "overview" | "portscan" | "docker";
const activeTab = ref<Tab>("overview");

const snapshot = ref<NetworkSnapshot | null>(null);
const docker = ref<DockerSnapshot | null>(null);

const hostsEditable = ref("");
const hostsSaving = ref(false);
const hostsSaveError = ref<string | null>(null);
const hostsSaveSuccess = ref(false);

const dnsEditable = ref("");
const dnsSaving = ref(false);
const dnsSaveError = ref<string | null>(null);
const dnsSaveSuccess = ref(false);

const firewallPortProto = ref("");
const firewallResult = ref<string | null>(null);
const firewallError = ref<string | null>(null);
const firewallBusy = ref(false);

onMounted(async () => {
  snapshot.value = await invoke<NetworkSnapshot>("get_network_snapshot");
  docker.value = await invoke<DockerSnapshot>("get_docker_snapshot");
  if (snapshot.value) {
    hostsEditable.value = snapshot.value.hosts_file;
    dnsEditable.value = snapshot.value.dns_servers.join("\n");
  }
});

async function saveHosts() {
  hostsSaving.value = true;
  hostsSaveError.value = null;
  hostsSaveSuccess.value = false;
  try {
    await invoke("write_hosts_file", { content: hostsEditable.value });
    hostsSaveSuccess.value = true;
    snapshot.value = await invoke<NetworkSnapshot>("get_network_snapshot");
  } catch (e) {
    hostsSaveError.value = String(e);
  } finally {
    hostsSaving.value = false;
  }
}

async function saveDns() {
  dnsSaving.value = true;
  dnsSaveError.value = null;
  dnsSaveSuccess.value = false;
  try {
    await invoke("set_dns_servers", { content: dnsEditable.value });
    dnsSaveSuccess.value = true;
    snapshot.value = await invoke<NetworkSnapshot>("get_network_snapshot");
  } catch (e) {
    dnsSaveError.value = String(e);
  } finally {
    dnsSaving.value = false;
  }
}

async function addFirewallRule() {
  firewallBusy.value = true;
  firewallError.value = null;
  firewallResult.value = null;
  try {
    firewallResult.value = await invoke<string>("add_firewall_rule", { portProto: firewallPortProto.value });
  } catch (e) {
    firewallError.value = String(e);
  } finally {
    firewallBusy.value = false;
  }
}

async function removeFirewallRule() {
  firewallBusy.value = true;
  firewallError.value = null;
  firewallResult.value = null;
  try {
    firewallResult.value = await invoke<string>("remove_firewall_rule", { portProto: firewallPortProto.value });
  } catch (e) {
    firewallError.value = String(e);
  } finally {
    firewallBusy.value = false;
  }
}

const scanHost = ref("127.0.0.1");
const scanPortsInput = ref("22,80,443,3000,8080");
const scanResults = ref<PortResult[]>([]);
const scanError = ref<string | null>(null);
const scanning = ref(false);

async function runScan() {
  scanning.value = true;
  scanError.value = null;
  try {
    const ports = scanPortsInput.value
      .split(",")
      .map((p) => parseInt(p.trim(), 10))
      .filter((p) => !Number.isNaN(p));
    scanResults.value = await invoke<PortResult[]>("scan_ports_cmd", { host: scanHost.value, ports });
  } catch (e) {
    scanError.value = String(e);
  } finally {
    scanning.value = false;
  }
}
</script>

<template>
  <div class="net-page">
    <h1>Réseau</h1>

    <div class="net-tabs">
      <button :class="{ active: activeTab === 'overview' }" @click="activeTab = 'overview'">Vue d'ensemble</button>
      <button :class="{ active: activeTab === 'portscan' }" @click="activeTab = 'portscan'">Scanner de ports</button>
      <button :class="{ active: activeTab === 'docker' }" @click="activeTab = 'docker'">Docker</button>
    </div>

    <section v-if="activeTab === 'overview' && snapshot" class="net-panel">
      <h2>Wi-Fi</h2>
      <div v-for="w in snapshot.wifi_networks" :key="w.ssid" class="net-row">
        <span>{{ w.ssid }}{{ w.connected ? " (connecté)" : "" }}</span>
        <span>{{ w.security }} · {{ w.signal_percent }}%</span>
      </div>

      <h2>Ports en écoute</h2>
      <div v-for="p in snapshot.listening_ports" :key="p.port" class="net-row">
        <span>{{ p.port }}</span>
        <span>{{ p.process ?? "?" }}</span>
      </div>

      <h2>Serveurs DNS</h2>
      <div v-for="d in snapshot.dns_servers" :key="d" class="net-row"><span>{{ d }}</span></div>

      <h2>/etc/hosts</h2>
      <pre class="net-hosts">{{ snapshot.hosts_file }}</pre>

      <h2>Modifier /etc/hosts</h2>
      <textarea v-model="hostsEditable" class="net-textarea" rows="8"></textarea>
      <div class="net-form-row">
        <button :disabled="hostsSaving" @click="saveHosts">{{ hostsSaving ? "Enregistrement..." : "Enregistrer" }}</button>
      </div>
      <div v-if="hostsSaveError" class="net-error">{{ hostsSaveError }}</div>
      <div v-if="hostsSaveSuccess" class="net-success">Fichier hosts mis à jour.</div>

      <h2>Modifier les serveurs DNS</h2>
      <textarea v-model="dnsEditable" class="net-textarea" rows="4" placeholder="nameserver 1.1.1.1"></textarea>
      <div class="net-form-row">
        <button :disabled="dnsSaving" @click="saveDns">{{ dnsSaving ? "Enregistrement..." : "Enregistrer" }}</button>
      </div>
      <div v-if="dnsSaveError" class="net-error">{{ dnsSaveError }}</div>
      <div v-if="dnsSaveSuccess" class="net-success">Configuration DNS mise à jour.</div>

      <h2>Règle de pare-feu</h2>
      <div class="net-form-row">
        <input v-model="firewallPortProto" class="net-input" placeholder="ex: 8080/tcp" />
        <button :disabled="firewallBusy" @click="addFirewallRule">Autoriser</button>
        <button :disabled="firewallBusy" @click="removeFirewallRule">Supprimer</button>
      </div>
      <div v-if="firewallError" class="net-error">{{ firewallError }}</div>
      <div v-if="firewallResult" class="net-success">Règle appliquée.</div>
    </section>

    <section v-else-if="activeTab === 'portscan'" class="net-panel">
      <div class="net-form-row">
        <input v-model="scanHost" class="net-input" placeholder="Hôte (ex: 127.0.0.1)" />
        <input v-model="scanPortsInput" class="net-input" placeholder="Ports, séparés par virgule" />
        <button :disabled="scanning" @click="runScan">{{ scanning ? "Scan..." : "Scanner" }}</button>
      </div>
      <div v-if="scanError" class="net-error">{{ scanError }}</div>
      <div v-for="r in scanResults" :key="r.port" class="net-row">
        <span>{{ r.port }}</span>
        <span :class="r.open ? 'net-open' : 'net-closed'">{{ r.open ? "ouvert" : "fermé" }}</span>
      </div>
    </section>

    <section v-else-if="activeTab === 'docker'" class="net-panel">
      <div v-if="!docker?.available" class="net-empty">Docker n'est pas disponible sur ce système.</div>
      <template v-else>
        <h2>Conteneurs</h2>
        <div v-for="c in docker.containers" :key="c.id" class="net-row">
          <span>{{ c.name }} ({{ c.image }})</span>
          <span>{{ c.status }}</span>
        </div>
        <h2>Images</h2>
        <div v-for="i in docker.images" :key="i.id" class="net-row">
          <span>{{ i.repository }}:{{ i.tag }}</span>
          <span>{{ i.size }}</span>
        </div>
      </template>
    </section>
  </div>
</template>

<style scoped>
.net-page { padding: 24px; color: var(--nx-text-primary); }
.net-tabs { display: flex; gap: 8px; margin: 16px 0; }
.net-tabs button { padding: 8px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-secondary); cursor: pointer; }
.net-tabs button.active { color: var(--nx-text-primary); border-color: var(--nx-accent-primary); }
.net-panel h2 { font-size: 14px; margin: 16px 0 6px; color: var(--nx-text-secondary); }
.net-row { display: flex; justify-content: space-between; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-border); }
.net-hosts { background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); border-radius: 8px; padding: 12px; font-size: 12px; overflow: auto; max-height: 200px; }
.net-textarea { width: 100%; padding: 10px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); font-family: monospace; font-size: 12px; margin-bottom: 8px; }
.net-success { margin-top: 10px; padding: 10px 14px; border-radius: 8px; background: color-mix(in srgb, var(--nx-accent-success) 15%, transparent); border: 1px solid var(--nx-accent-success); }
.net-form-row { display: flex; gap: 10px; align-items: center; }
.net-input { flex: 1; padding: 8px 10px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); }
.net-error { margin-top: 10px; padding: 10px 14px; border-radius: 8px; background: color-mix(in srgb, var(--nx-accent-danger) 15%, transparent); border: 1px solid var(--nx-accent-danger); }
.net-open { color: var(--nx-accent-success); }
.net-closed { color: var(--nx-text-secondary); }
.net-empty { color: var(--nx-text-secondary); margin-top: 12px; }
</style>
