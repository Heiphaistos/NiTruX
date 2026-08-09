<!-- src/pages/ProcessesPage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface ProcessInfo { pid: number; name: string; cpu_percent: number; memory_bytes: number }
interface AutostartEntry { name: string }

const processes = ref<ProcessInfo[] | null>(null);
const services = ref<string[] | null>(null);
const autostart = ref<AutostartEntry[] | null>(null);
const scheduledTasks = ref<string[] | null>(null);
const processFilter = ref("");

const processesError = ref<string | null>(null);
const servicesError = ref<string | null>(null);
const autostartError = ref<string | null>(null);
const scheduledTasksError = ref<string | null>(null);

onMounted(async () => {
  // 4 independent sources (each backed by an infallible Vec<T>-returning
  // Rust command, never Result) -- one failing at the IPC layer itself
  // (rare but real, e.g. a Tauri-side panic) must not blank or skip the
  // other 3, matching the same "each source degrades independently"
  // fix applied to NetworkPage.vue (cycle 387) and InstalledSoftwarePage.vue.
  try {
    processes.value = await invoke<ProcessInfo[]>("get_processes");
  } catch (e) {
    processesError.value = String(e);
  }
  try {
    services.value = await invoke<string[]>("get_systemd_services");
  } catch (e) {
    servicesError.value = String(e);
  }
  try {
    autostart.value = await invoke<AutostartEntry[]>("get_autostart_entries");
  } catch (e) {
    autostartError.value = String(e);
  }
  try {
    scheduledTasks.value = await invoke<string[]>("get_scheduled_tasks");
  } catch (e) {
    scheduledTasksError.value = String(e);
  }
});

const filteredProcesses = computed(() =>
  (processes.value ?? []).filter((p) => p.name.toLowerCase().includes(processFilter.value.toLowerCase())),
);

function bytesToMb(bytes: number): string {
  return (bytes / 1024 / 1024).toFixed(1);
}
</script>

<template>
  <div class="proc-page">
    <NxSectionHeader title="Processus & services" description="Processus en cours, services, démarrage automatique et tâches planifiées." />

    <NxCard>
      <NxSectionHeader title="Processus" />
      <NxCard v-if="processesError" danger>{{ processesError }}</NxCard>
      <NxInput v-model="processFilter" placeholder="Filtrer par nom..." aria-label="Filtrer les processus par nom" />
      <div v-if="processes && filteredProcesses.length === 0" class="proc-empty">Aucun processus trouvé.</div>
      <div v-for="p in filteredProcesses" :key="p.pid" class="proc-row">
        <span>{{ p.name }} ({{ p.pid }})</span>
        <span>{{ p.cpu_percent.toFixed(1) }}% · {{ bytesToMb(p.memory_bytes) }} Mo</span>
      </div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Services systemd" />
      <NxCard v-if="servicesError" danger>{{ servicesError }}</NxCard>
      <div v-if="services && services.length === 0" class="proc-empty">Aucun service systemd trouvé.</div>
      <div v-for="s in services ?? []" :key="s" class="proc-row">{{ s }}</div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Démarrage automatique" />
      <NxCard v-if="autostartError" danger>{{ autostartError }}</NxCard>
      <div v-if="autostart && autostart.length === 0" class="proc-empty">Aucune entrée de démarrage automatique.</div>
      <div v-for="a in autostart ?? []" :key="a.name" class="proc-row">{{ a.name }}</div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Tâches planifiées" />
      <NxCard v-if="scheduledTasksError" danger>{{ scheduledTasksError }}</NxCard>
      <div v-if="scheduledTasks && scheduledTasks.length === 0" class="proc-empty">Aucune tâche planifiée.</div>
      <div v-for="t in scheduledTasks ?? []" :key="t" class="proc-row">{{ t }}</div>
    </NxCard>
  </div>
</template>

<style scoped>
.proc-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.proc-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 4px 0; font-size: 13px; }
.proc-empty { color: var(--nx-text-secondary); font-size: 13px; }
</style>
