<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface LogEntry { priority: number; message: string; unit: string }

const logs = ref<LogEntry[]>([]);
const error = ref<string | null>(null);

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
    <h1>Journaux système</h1>
    <div class="logs-error" v-if="error">
      Impossible de récupérer les journaux système : {{ error }}
    </div>
    <div class="logs-list" v-if="logs.length">
      <div v-for="(log, i) in logs" :key="i" class="log-entry" :class="priorityClass(log.priority)">
        <span class="log-unit">{{ log.unit }}</span>
        <span class="log-message">{{ log.message }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.logs-page { padding: 24px; color: var(--nx-text-primary); }
.logs-error { margin-top: 16px; padding: 12px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-secondary); }
.logs-list { margin-top: 16px; font-family: monospace; font-size: 12px; display: grid; gap: 2px; max-height: 70vh; overflow: auto; }
.log-entry { display: flex; gap: 10px; padding: 4px 8px; border-radius: 4px; }
.log-entry.log-error { background: color-mix(in srgb, var(--nx-accent-danger) 15%, transparent); }
.log-entry.log-warning { background: color-mix(in srgb, var(--nx-accent-warning) 15%, transparent); }
.log-unit { color: var(--nx-text-secondary); flex-shrink: 0; min-width: 120px; }
</style>
