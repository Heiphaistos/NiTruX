<!-- src/pages/AdminHistoryPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface AdminAction { timestamp_ms: number; action: string; ok: boolean; detail: string }

const entries = ref<AdminAction[] | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    entries.value = await invoke<AdminAction[]>("get_admin_history");
  } catch (e) {
    error.value = String(e);
  }
});

const when = (ms: number) => new Date(ms).toLocaleString("fr-FR");
</script>

<template>
  <div class="ah-page">
    <NxSectionHeader title="Actions administrateur" description="Tout ce que NiTruX a lancé avec les droits root, du plus récent au plus ancien." />
    <NxCard v-if="error" danger>{{ error }}</NxCard>
    <div v-else-if="entries && entries.length === 0" class="ah-empty">Aucune action administrateur pour l'instant.</div>
    <NxCard v-else-if="entries">
      <div class="ah-scroll">
        <table class="ah-table">
          <thead><tr><th>Date</th><th>Action</th><th>Résultat</th></tr></thead>
          <tbody>
            <tr v-for="(e, i) in entries" :key="i">
              <td>{{ when(e.timestamp_ms) }}</td>
              <td><code>{{ e.action }}</code></td>
              <td>
                <NxBadge :status="e.ok ? 'success' : 'danger'">{{ e.ok ? "réussie" : "échec" }}</NxBadge>
                <div v-if="e.detail" class="ah-detail">{{ e.detail }}</div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.ah-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.ah-empty { color: var(--nx-text-secondary); }
.ah-scroll { overflow-x: auto; }
.ah-table { width: 100%; min-width: 520px; border-collapse: collapse; font-size: 13px; }
.ah-table th { text-align: left; color: var(--nx-text-secondary); border-bottom: 1px solid var(--nx-style-border-color); padding: 8px; }
.ah-table td { padding: 8px; border-bottom: 1px solid var(--nx-style-border-color); vertical-align: top; }
.ah-detail { margin-top: 4px; font-size: 12px; color: var(--nx-text-secondary); }
</style>
