<!-- src/pages/TroubleshootPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

const TROUBLESHOOT_ACTIONS: { id: string; label: string }[] = [
  { id: "fix-broken", label: "Réparer les paquets cassés" },
  { id: "restart-network", label: "Redémarrer le réseau" },
  { id: "clean-cache", label: "Vider le cache du gestionnaire de paquets" },
  { id: "vacuum-logs", label: "Purger les journaux système de plus de 7 jours" },
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
</script>

<template>
  <div class="ts-page">
    <NxSectionHeader title="Dépannage" description="Actions de réparation courantes." />

    <NxCard>
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
.ts-form-row { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; }
.ts-action-label { flex: 1; }
</style>
