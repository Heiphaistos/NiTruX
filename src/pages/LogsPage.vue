<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface LogEntry { priority: number; message: string; unit: string }

const logs = ref<LogEntry[]>([]);
const error = ref<string | null>(null);
// 200 entries with no text filter is unusable to scan for one specific
// event -- ProcessesPage/InstalledSoftwarePage/UninstallerPage all have
// an NxInput filter for comparably-sized lists; mirrored here rather
// than inventing a different mechanism for the same underlying gap.
const logFilter = ref("");

const filteredLogs = computed(() => {
  const q = logFilter.value.toLowerCase();
  if (q === "") return logs.value;
  return logs.value.filter((log) => log.unit.toLowerCase().includes(q) || log.message.toLowerCase().includes(q));
});

function priorityClass(priority: number): string {
  if (priority <= 3) return "log-error";
  if (priority <= 4) return "log-warning";
  return "log-info";
}

onMounted(async () => {
  try {
    logs.value = await invoke<LogEntry[]>("get_recent_logs", { limit: 200 });
    error.value = null;
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  }
});
</script>

<template>
  <div class="logs-page">
    <NxSectionHeader title="Journaux" description="Journaux système récents (journald)." />
    <NxCard v-if="error" danger>
      Impossible de récupérer les journaux système : {{ error }}
    </NxCard>
    <NxInput v-if="logs.length" v-model="logFilter" placeholder="Filtrer par unité ou message..." aria-label="Filtrer les journaux" />
    <NxCard v-if="logs.length" class="logs-card">
      <div v-if="filteredLogs.length === 0" class="logs-empty">Aucun journal ne correspond à ce filtre.</div>
      <div class="logs-list">
        <div v-for="(log, i) in filteredLogs" :key="i" class="log-entry" :class="priorityClass(log.priority)">
          <span class="log-unit">{{ log.unit }}</span>
          <span class="log-message">{{ log.message }}</span>
        </div>
      </div>
    </NxCard>
    <NxCard v-else-if="!error" class="logs-empty">Aucun journal récent trouvé.</NxCard>
  </div>
</template>

<style scoped>
.logs-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.logs-empty { color: var(--nx-text-secondary); font-size: 13px; }
.logs-card { padding: 8px; }
.logs-list { font-family: monospace; font-size: 12px; display: grid; gap: 2px; max-height: 70vh; overflow: auto; }
.log-entry { display: flex; gap: 10px; padding: 4px 8px; border-radius: 4px; }
.log-entry.log-error { background: color-mix(in srgb, var(--nx-accent-danger) 15%, transparent); }
.log-entry.log-warning { background: color-mix(in srgb, var(--nx-accent-warning) 15%, transparent); }
.log-unit { color: var(--nx-text-secondary); flex-shrink: 0; min-width: 120px; }
</style>
