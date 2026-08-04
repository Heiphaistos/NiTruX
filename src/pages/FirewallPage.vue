<!-- src/pages/FirewallPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface FirewallStatus { active: boolean; rules: string[] }

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
</script>

<template>
  <div class="fw-page">
    <NxSectionHeader title="Pare-feu" description="État et règles UFW actives." />
    <NxCard v-if="firewallError" danger>{{ firewallError }}</NxCard>
    <template v-else-if="firewall">
      <NxBadge :status="firewall.active ? 'success' : 'warning'">
        UFW {{ firewall.active ? "actif" : "inactif" }}
      </NxBadge>
      <NxCard v-if="firewall.active" class="fw-rules">
        <div v-if="firewall.rules.length === 0" class="fw-empty">Aucune règle configurée.</div>
        <div v-for="(r, i) in firewall.rules" :key="i" class="fw-row">{{ r }}</div>
      </NxCard>
    </template>
  </div>
</template>

<style scoped>
.fw-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.fw-rules { padding: 4px 16px; }
.fw-row { padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
.fw-row:last-child { border-bottom: none; }
.fw-empty { color: var(--nx-text-secondary); font-size: 13px; }
</style>
