<!-- src/pages/DnsSwitcherPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

const PRESETS = [
  { label: "Cloudflare (1.1.1.1)", servers: ["1.1.1.1", "1.0.0.1"] },
  { label: "Google (8.8.8.8)", servers: ["8.8.8.8", "8.8.4.4"] },
  { label: "Quad9 (9.9.9.9)", servers: ["9.9.9.9", "149.112.112.112"] },
];

const currentDns = ref<string[]>([]);
const manualDns = ref("");
const applying = ref(false);
const applyError = ref<string | null>(null);
const applySuccess = ref(false);

async function refresh() {
  const snapshot = await invoke<{ dns_servers: string[] }>("get_network_snapshot");
  currentDns.value = snapshot.dns_servers;
}

onMounted(refresh);

async function apply(content: string) {
  applying.value = true;
  applyError.value = null;
  applySuccess.value = false;
  try {
    await invoke("set_dns_servers", { content });
    applySuccess.value = true;
    await refresh();
  } catch (e) {
    applyError.value = String(e);
  } finally {
    applying.value = false;
  }
}
</script>

<template>
  <div class="dns-page">
    <NxSectionHeader title="DNS Switcher" :description="`Serveurs actuels : ${currentDns.join(', ') || 'aucun'}`" />

    <NxCard v-if="applyError" danger>{{ applyError }}</NxCard>
    <NxBadge v-if="applySuccess" status="success">DNS mis à jour.</NxBadge>

    <NxCard class="dns-presets">
      <NxButton v-for="p in PRESETS" :key="p.label" :disabled="applying" @click="apply(p.servers.join('\n'))">
        {{ p.label }}
      </NxButton>
    </NxCard>

    <NxCard class="dns-manual">
      <textarea v-model="manualDns" class="dns-textarea" rows="4" placeholder="Un serveur DNS par ligne..."></textarea>
      <NxButton :disabled="applying || manualDns === ''" @click="apply(manualDns)">Appliquer</NxButton>
    </NxCard>
  </div>
</template>

<style scoped>
.dns-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.dns-presets { display: flex; gap: 10px; flex-wrap: wrap; }
.dns-manual { display: flex; flex-direction: column; gap: 10px; }
.dns-textarea { width: 100%; padding: 10px; border-radius: var(--nx-style-radius); border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-primary); font-family: monospace; font-size: 12px; }
</style>
