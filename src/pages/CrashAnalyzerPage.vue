<!-- src/pages/CrashAnalyzerPage.vue -->
<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

type CrashKind = "KernelPanic" | "OomKill" | "Segfault" | "KernelError";
interface CrashEvent { kind: CrashKind; message: string; unit: string }

const events = ref<CrashEvent[] | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    events.value = await invoke<CrashEvent[]>("get_crash_events");
  } catch (e) {
    error.value = String(e);
  }
});

const kindLabels: Record<CrashKind, string> = {
  KernelPanic: "Panique noyau",
  OomKill: "Manque de mémoire",
  Segfault: "Erreur de segmentation",
  KernelError: "Erreur noyau critique",
};

function statusFor(kind: CrashKind): "danger" | "warning" {
  return kind === "Segfault" ? "warning" : "danger";
}

const sortedEvents = computed(() => [...(events.value ?? [])].reverse());
</script>

<template>
  <div class="crash-page">
    <NxSectionHeader
      title="Analyseur de pannes"
      description="Paniques noyau, manques de mémoire (OOM) et erreurs de segmentation récentes, extraits des journaux système."
    />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <NxCard v-if="events && events.length === 0" class="crash-empty">
      Aucune panne détectée dans les journaux récents.
    </NxCard>

    <NxCard v-for="(e, i) in sortedEvents" :key="i" class="crash-row">
      <div class="crash-info">
        <NxBadge :status="statusFor(e.kind)">{{ kindLabels[e.kind] }}</NxBadge>
        <span class="crash-unit">{{ e.unit }}</span>
      </div>
      <span class="crash-message">{{ e.message }}</span>
    </NxCard>
  </div>
</template>

<style scoped>
.crash-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.crash-empty { color: var(--nx-text-secondary); font-size: 13px; }
.crash-row { display: flex; flex-direction: column; gap: 6px; }
.crash-info { display: flex; align-items: center; gap: 10px; }
.crash-unit { color: var(--nx-text-secondary); font-size: 12px; }
.crash-message { font-family: monospace; font-size: 12px; word-break: break-word; }
</style>
