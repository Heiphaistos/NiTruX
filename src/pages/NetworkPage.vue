<!-- src/pages/NetworkPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSelect from "@/components/ui/NxSelect.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface WifiNetwork { ssid: string; security: string; signal_percent: number; connected: boolean }
interface ListeningPort { port: number; process: string | null; protocol: string | null }
interface RouteEntry { destination: string; gateway: string | null; interface: string | null; metric: number | null }
interface ArpEntry { ip: string; mac: string | null; interface: string | null; state: string }
interface NetworkSnapshot { wifi_networks: WifiNetwork[]; listening_ports: ListeningPort[]; dns_servers: string[]; hosts_file: string; routes: RouteEntry[]; arp_entries: ArpEntry[] }
interface PortResult { port: number; open: boolean }
interface PingSummary {
  packets_sent: number;
  packets_received: number;
  loss_percent: number;
  min_ms: number | null;
  avg_ms: number | null;
  max_ms: number | null;
}
interface Container { id: string; image: string; name: string; status: string }
interface DockerImageInfo { id: string; repository: string; tag: string; size: string }
interface DockerSnapshot { available: boolean; installed: boolean; error: string | null; containers: Container[]; images: DockerImageInfo[] }
interface NetworkInterface { name: string; mac_address: string | null; rx_bytes_per_sec: number; tx_bytes_per_sec: number }

type Tab = "overview" | "portscan" | "ping" | "dns" | "docker";
const activeTab = ref<Tab>("overview");

const snapshot = ref<NetworkSnapshot | null>(null);
const docker = ref<DockerSnapshot | null>(null);
const interfaces = ref<NetworkInterface[] | null>(null);

function formatThroughput(bytesPerSec: number): string {
  return `${(bytesPerSec / 1024).toFixed(1)} Ko/s`;
}

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
  interfaces.value = await invoke<NetworkInterface[]>("get_network_interfaces");
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

// The backend's `ports` parameter is a Vec<u16> -- a value outside 0-65535
// (parseInt has no inherent range limit, unlike Number.isNaN alone
// catching non-numeric/empty segments) fails Tauri IPC's own JSON
// deserialization before scan_ports_cmd's body ever runs, surfacing a raw,
// cryptic "invalid value: integer `99999`, expected u16" instead of a
// message the user can act on -- the exact same class of bug already
// documented and guarded against in FileToolsPage.vue's minSizeMb
// validation, generalized here for a value going into an array instead of
// a single field.
function isValidPort(p: number): boolean {
  return !Number.isNaN(p) && p >= 0 && p <= 65535;
}

async function runScan() {
  scanning.value = true;
  scanError.value = null;
  try {
    const ports = scanPortsInput.value
      .split(",")
      .map((p) => parseInt(p.trim(), 10))
      .filter(isValidPort);
    scanResults.value = await invoke<PortResult[]>("scan_ports_cmd", { host: scanHost.value, ports });
  } catch (e) {
    scanError.value = String(e);
  } finally {
    scanning.value = false;
  }
}

const pingHost = ref("8.8.8.8");
const pingResult = ref<PingSummary | null>(null);
const pingError = ref<string | null>(null);
const pinging = ref(false);

async function runPing() {
  pinging.value = true;
  pingError.value = null;
  pingResult.value = null;
  try {
    pingResult.value = await invoke<PingSummary>("ping_host", { host: pingHost.value });
  } catch (e) {
    pingError.value = String(e);
  } finally {
    pinging.value = false;
  }
}

const DNS_QUERY_TYPE_OPTIONS = [
  { value: "A", label: "A" },
  { value: "AAAA", label: "AAAA" },
  { value: "MX", label: "MX" },
  { value: "TXT", label: "TXT" },
  { value: "CNAME", label: "CNAME" },
  { value: "NS", label: "NS" },
];

const dnsLookupHost = ref("");
const dnsLookupType = ref("A");
const dnsLookupResults = ref<string[] | null>(null);
const dnsLookupError = ref<string | null>(null);
const dnsLookingUp = ref(false);

async function runDnsLookup() {
  dnsLookingUp.value = true;
  dnsLookupError.value = null;
  dnsLookupResults.value = null;
  try {
    dnsLookupResults.value = await invoke<string[]>("dns_lookup", { host: dnsLookupHost.value, queryType: dnsLookupType.value });
  } catch (e) {
    dnsLookupError.value = String(e);
  } finally {
    dnsLookingUp.value = false;
  }
}
</script>

<template>
  <div class="net-page">
    <NxSectionHeader title="Réseau" />

    <div class="net-tabs">
      <button :class="{ active: activeTab === 'overview' }" @click="activeTab = 'overview'">Vue d'ensemble</button>
      <button :class="{ active: activeTab === 'portscan' }" @click="activeTab = 'portscan'">Scanner de ports</button>
      <button :class="{ active: activeTab === 'ping' }" @click="activeTab = 'ping'">Ping</button>
      <button :class="{ active: activeTab === 'dns' }" @click="activeTab = 'dns'">Recherche DNS</button>
      <button :class="{ active: activeTab === 'docker' }" @click="activeTab = 'docker'">Docker</button>
    </div>

    <template v-if="activeTab === 'overview' && snapshot">
      <NxCard v-if="interfaces">
        <NxSectionHeader title="Cartes réseau" />
        <div v-if="interfaces.length === 0" class="net-empty">Aucune carte réseau détectée.</div>
        <div v-for="i in interfaces" :key="i.name" class="net-row net-iface-row">
          <span>{{ i.name }}</span>
          <span>{{ i.mac_address ?? "MAC inconnue" }}</span>
          <span>↓ {{ formatThroughput(i.rx_bytes_per_sec) }} · ↑ {{ formatThroughput(i.tx_bytes_per_sec) }}</span>
        </div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Wi-Fi" />
        <div v-for="(w, wi) in snapshot.wifi_networks" :key="`${w.ssid}-${wi}`" class="net-row">
          <span>{{ w.ssid }}{{ w.connected ? " (connecté)" : "" }}</span>
          <span>{{ w.security }} · {{ w.signal_percent }}%</span>
        </div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Ports en écoute" />
        <div v-for="(p, pi) in snapshot.listening_ports" :key="`${p.port}-${p.protocol}-${pi}`" class="net-row">
          <span>{{ p.port }} <span v-if="p.protocol" class="net-proto">{{ p.protocol.toUpperCase() }}</span></span>
          <span>{{ p.process ?? "?" }}</span>
        </div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Table de routage" />
        <div v-if="snapshot.routes.length === 0" class="net-empty">Aucune route détectée.</div>
        <div v-for="(r, ri) in snapshot.routes" :key="`${r.destination}-${ri}`" class="net-row">
          <span class="net-dns-record">{{ r.destination }}</span>
          <span>
            <template v-if="r.gateway">via {{ r.gateway }} </template>
            <template v-if="r.interface">dev {{ r.interface }} </template>
            <template v-if="r.metric !== null">metric {{ r.metric }}</template>
          </span>
        </div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Table ARP" />
        <div v-if="snapshot.arp_entries.length === 0" class="net-empty">Aucune entrée ARP détectée.</div>
        <div v-for="(a, ai) in snapshot.arp_entries" :key="`${a.ip}-${ai}`" class="net-row">
          <span class="net-dns-record">{{ a.ip }}</span>
          <span>{{ a.mac ?? "(inconnue)" }} · {{ a.interface ?? "?" }} · {{ a.state }}</span>
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
        <textarea v-model="dnsEditable" class="net-textarea" rows="4" placeholder="nameserver 1.1.1.1" aria-label="Serveurs DNS"></textarea>
        <NxButton :disabled="dnsSaving" @click="saveDns">{{ dnsSaving ? "Enregistrement..." : "Enregistrer" }}</NxButton>
        <NxCard v-if="dnsSaveError" danger>{{ dnsSaveError }}</NxCard>
        <div v-if="dnsSaveSuccess" class="net-success">Configuration DNS mise à jour.</div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Règle de pare-feu" />
        <div class="net-form-row">
          <NxInput v-model="firewallPortProto" placeholder="ex: 8080/tcp" aria-label="Port et protocole" />
          <NxButton :disabled="firewallBusy" @click="addFirewallRule">Autoriser</NxButton>
          <NxButton :disabled="firewallBusy" @click="removeFirewallRule">Supprimer</NxButton>
        </div>
        <NxCard v-if="firewallError" danger>{{ firewallError }}</NxCard>
        <div v-if="firewallResult" class="net-success">Règle appliquée.</div>
      </NxCard>
    </template>

    <NxCard v-else-if="activeTab === 'portscan'">
      <div class="net-form-row">
        <NxInput v-model="scanHost" placeholder="Hôte (ex: 127.0.0.1)" aria-label="Hôte à scanner" />
        <NxInput v-model="scanPortsInput" placeholder="Ports, séparés par virgule" aria-label="Ports à scanner" />
        <NxButton :disabled="scanning" @click="runScan">{{ scanning ? "Scan..." : "Scanner" }}</NxButton>
      </div>
      <NxCard v-if="scanError" danger>{{ scanError }}</NxCard>
      <div v-for="(r, ri) in scanResults" :key="`${r.port}-${ri}`" class="net-row">
        <span>{{ r.port }}</span>
        <span :class="r.open ? 'net-open' : 'net-closed'">{{ r.open ? "ouvert" : "fermé" }}</span>
      </div>
    </NxCard>

    <NxCard v-else-if="activeTab === 'ping'">
      <div class="net-form-row">
        <NxInput v-model="pingHost" placeholder="Hôte (ex: 8.8.8.8)" aria-label="Hôte à pinger" />
        <NxButton :disabled="pinging" @click="runPing">{{ pinging ? "Ping..." : "Pinger" }}</NxButton>
      </div>
      <NxCard v-if="pingError" danger>{{ pingError }}</NxCard>
      <div v-if="pingResult" class="net-ping-result">
        <div class="net-row">
          <span>Paquets</span>
          <span>{{ pingResult.packets_received }} / {{ pingResult.packets_sent }} reçus ({{ pingResult.loss_percent }}% de perte)</span>
        </div>
        <div v-if="pingResult.avg_ms !== null" class="net-row">
          <span>Latence</span>
          <span>min {{ pingResult.min_ms }} ms · moy {{ pingResult.avg_ms }} ms · max {{ pingResult.max_ms }} ms</span>
        </div>
      </div>
    </NxCard>

    <NxCard v-else-if="activeTab === 'dns'">
      <div class="net-form-row">
        <NxInput v-model="dnsLookupHost" placeholder="Nom d'hôte (ex: example.com)" aria-label="Hôte à résoudre" />
        <NxSelect v-model="dnsLookupType" :options="DNS_QUERY_TYPE_OPTIONS" aria-label="Type d'enregistrement DNS" />
        <NxButton :disabled="dnsLookingUp" @click="runDnsLookup">{{ dnsLookingUp ? "Recherche..." : "Rechercher" }}</NxButton>
      </div>
      <NxCard v-if="dnsLookupError" danger>{{ dnsLookupError }}</NxCard>
      <div v-if="dnsLookupResults && dnsLookupResults.length === 0" class="net-empty">Aucun enregistrement {{ dnsLookupType }} trouvé pour cet hôte.</div>
      <div v-for="(r, ri) in dnsLookupResults ?? []" :key="`${r}-${ri}`" class="net-row">
        <span class="net-dns-record">{{ r }}</span>
      </div>
    </NxCard>

    <NxCard v-else-if="activeTab === 'docker'">
      <div v-if="!docker?.available && !docker?.installed" class="net-empty">Docker n'est pas installé sur ce système.</div>
      <div v-else-if="!docker?.available" class="net-empty">
        Docker est installé mais injoignable{{ docker?.error ? " : " + docker.error : "" }}. Vérifiez que le démon est démarré et que votre utilisateur a les droits nécessaires.
      </div>
      <template v-else>
        <NxSectionHeader title="Conteneurs" />
        <div v-for="c in docker.containers" :key="c.id" class="net-row">
          <span>{{ c.name }} ({{ c.image }})</span>
          <span>{{ c.status }}</span>
        </div>
        <div v-if="docker.containers.length === 0" class="net-empty">Aucun conteneur.</div>
        <NxSectionHeader title="Images" />
        <div v-for="(i, ii) in docker.images" :key="`${i.id}-${ii}`" class="net-row">
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
.net-proto { font-size: 10px; font-weight: 600; color: var(--nx-text-secondary); background: var(--nx-bg-elevated); border-radius: 4px; padding: 1px 5px; margin-left: 4px; }
.net-dns-record { font-family: monospace; word-break: break-all; }
.net-textarea { width: 100%; padding: 10px; border-radius: var(--nx-style-radius); border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-primary); font-family: monospace; font-size: 12px; margin-bottom: 8px; }
.net-success { margin-top: 10px; padding: 10px 14px; border-radius: var(--nx-style-radius); background: color-mix(in srgb, var(--nx-accent-success) 15%, transparent); border: 1px solid var(--nx-accent-success); }
.net-form-row { display: flex; gap: 10px; align-items: center; }
.net-open { color: var(--nx-accent-success); }
.net-closed { color: var(--nx-text-secondary); }
.net-empty { color: var(--nx-text-secondary); }
</style>
