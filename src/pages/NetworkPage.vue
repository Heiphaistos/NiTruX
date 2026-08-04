<!-- src/pages/NetworkPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

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
    dnsEditable.value = snapshot.value.dns_servers.map((ip) => `nameserver ${ip}`).join("\n");
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
    <NxSectionHeader title="Réseau" />

    <div class="net-tabs">
      <button :class="{ active: activeTab === 'overview' }" @click="activeTab = 'overview'">Vue d'ensemble</button>
      <button :class="{ active: activeTab === 'portscan' }" @click="activeTab = 'portscan'">Scanner de ports</button>
      <button :class="{ active: activeTab === 'docker' }" @click="activeTab = 'docker'">Docker</button>
    </div>

    <template v-if="activeTab === 'overview' && snapshot">
      <NxCard>
        <NxSectionHeader title="Wi-Fi" />
        <div v-for="w in snapshot.wifi_networks" :key="w.ssid" class="net-row">
          <span>{{ w.ssid }}{{ w.connected ? " (connecté)" : "" }}</span>
          <span>{{ w.security }} · {{ w.signal_percent }}%</span>
        </div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Ports en écoute" />
        <div v-for="p in snapshot.listening_ports" :key="p.port" class="net-row">
          <span>{{ p.port }}</span>
          <span>{{ p.process ?? "?" }}</span>
        </div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Modifier /etc/hosts" />
        <textarea v-model="hostsEditable" class="net-textarea" rows="8"></textarea>
        <NxButton :disabled="hostsSaving" @click="saveHosts">{{ hostsSaving ? "Enregistrement..." : "Enregistrer" }}</NxButton>
        <NxCard v-if="hostsSaveError" danger>{{ hostsSaveError }}</NxCard>
        <div v-if="hostsSaveSuccess" class="net-success">Fichier hosts mis à jour.</div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Modifier les serveurs DNS" />
        <textarea v-model="dnsEditable" class="net-textarea" rows="4" placeholder="nameserver 1.1.1.1"></textarea>
        <NxButton :disabled="dnsSaving" @click="saveDns">{{ dnsSaving ? "Enregistrement..." : "Enregistrer" }}</NxButton>
        <NxCard v-if="dnsSaveError" danger>{{ dnsSaveError }}</NxCard>
        <div v-if="dnsSaveSuccess" class="net-success">Configuration DNS mise à jour.</div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Règle de pare-feu" />
        <div class="net-form-row">
          <NxInput v-model="firewallPortProto" placeholder="ex: 8080/tcp" />
          <NxButton :disabled="firewallBusy" @click="addFirewallRule">Autoriser</NxButton>
          <NxButton :disabled="firewallBusy" @click="removeFirewallRule">Supprimer</NxButton>
        </div>
        <NxCard v-if="firewallError" danger>{{ firewallError }}</NxCard>
        <div v-if="firewallResult" class="net-success">Règle appliquée.</div>
      </NxCard>
    </template>

    <NxCard v-else-if="activeTab === 'portscan'">
      <div class="net-form-row">
        <NxInput v-model="scanHost" placeholder="Hôte (ex: 127.0.0.1)" />
        <NxInput v-model="scanPortsInput" placeholder="Ports, séparés par virgule" />
        <NxButton :disabled="scanning" @click="runScan">{{ scanning ? "Scan..." : "Scanner" }}</NxButton>
      </div>
      <NxCard v-if="scanError" danger>{{ scanError }}</NxCard>
      <div v-for="r in scanResults" :key="r.port" class="net-row">
        <span>{{ r.port }}</span>
        <span :class="r.open ? 'net-open' : 'net-closed'">{{ r.open ? "ouvert" : "fermé" }}</span>
      </div>
    </NxCard>

    <NxCard v-else-if="activeTab === 'docker'">
      <div v-if="!docker?.available" class="net-empty">Docker n'est pas disponible sur ce système.</div>
      <template v-else>
        <NxSectionHeader title="Conteneurs" />
        <div v-for="c in docker.containers" :key="c.id" class="net-row">
          <span>{{ c.name }} ({{ c.image }})</span>
          <span>{{ c.status }}</span>
        </div>
        <div v-if="docker.containers.length === 0" class="net-empty">Aucun conteneur.</div>
        <NxSectionHeader title="Images" />
        <div v-for="i in docker.images" :key="i.id" class="net-row">
          <span>{{ i.repository }}:{{ i.tag }}</span>
          <span>{{ i.size }}</span>
        </div>
        <div v-if="docker.images.length === 0" class="net-empty">Aucune image.</div>
      </template>
    </NxCard>
  </div>
</template>

<style scoped>
.net-page { padding: 24px; display: flex; flex-direction: column; gap: 14px; }
.net-tabs { display: flex; gap: 8px; }
.net-tabs button { padding: 8px 14px; border-radius: var(--nx-style-radius); border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-secondary); cursor: pointer; font: inherit; }
.net-tabs button.active { color: var(--nx-text-primary); font-weight: 600; }
.net-row { display: flex; justify-content: space-between; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
.net-textarea { width: 100%; padding: 10px; border-radius: var(--nx-style-radius); border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-primary); font-family: monospace; font-size: 12px; margin-bottom: 8px; }
.net-success { margin-top: 10px; padding: 10px 14px; border-radius: var(--nx-style-radius); background: color-mix(in srgb, var(--nx-accent-success) 15%, transparent); border: 1px solid var(--nx-accent-success); }
.net-form-row { display: flex; gap: 10px; align-items: center; }
.net-open { color: var(--nx-accent-success); }
.net-closed { color: var(--nx-text-secondary); }
.net-empty { color: var(--nx-text-secondary); }
</style>
