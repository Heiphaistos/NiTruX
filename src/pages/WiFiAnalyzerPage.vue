<!-- src/pages/WiFiAnalyzerPage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface WifiNetwork { ssid: string; security: string; signal_percent: number; connected: boolean }
interface NetworkSnapshot { wifi_networks: WifiNetwork[] }

const networks = ref<WifiNetwork[]>([]);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    const snapshot = await invoke<NetworkSnapshot>("get_network_snapshot");
    networks.value = snapshot.wifi_networks;
  } catch (e) {
    error.value = String(e);
  }
});

const sortedNetworks = computed(() => [...networks.value].sort((a, b) => b.signal_percent - a.signal_percent));

function securityStatus(security: string): "success" | "warning" | "danger" {
  if (security === "") return "danger";
  if (security.includes("WEP")) return "warning";
  return "success";
}
</script>

<template>
  <div class="wifi-page">
    <NxSectionHeader title="WiFi Analyzer" description="Réseaux Wi-Fi visibles, classés par force de signal." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <NxCard v-for="(net, i) in sortedNetworks" :key="`${net.ssid}-${i}`" class="wifi-row">
      <div class="wifi-info">
        <span class="wifi-ssid">{{ net.ssid }}</span>
        <span v-if="net.connected" class="wifi-connected">(connecté)</span>
        <NxBadge :status="securityStatus(net.security)">{{ net.security || "ouvert" }}</NxBadge>
      </div>
      <div class="wifi-signal-bar">
        <div class="wifi-signal-fill" :style="{ width: `${net.signal_percent}%` }"></div>
      </div>
      <span class="wifi-signal-label">{{ net.signal_percent }}%</span>
    </NxCard>
    <NxCard v-if="!error && sortedNetworks.length === 0" class="wifi-empty">Aucun réseau Wi-Fi détecté.</NxCard>
  </div>
</template>

<style scoped>
.wifi-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.wifi-row { display: flex; align-items: center; gap: 14px; }
.wifi-info { display: flex; flex-direction: column; gap: 6px; min-width: 160px; }
.wifi-ssid { font-weight: 600; color: var(--nx-text-primary); }
.wifi-connected { font-size: 11px; color: var(--nx-text-secondary); }
.wifi-signal-bar { flex: 1; height: 6px; border-radius: 3px; background: color-mix(in srgb, var(--nx-accent-primary) 15%, transparent); overflow: hidden; }
.wifi-signal-fill { height: 100%; background: var(--nx-accent-primary); border-radius: 3px; }
.wifi-signal-label { font-size: 12px; color: var(--nx-text-secondary); min-width: 36px; text-align: right; }
.wifi-empty { color: var(--nx-text-secondary); font-size: 13px; }
</style>
