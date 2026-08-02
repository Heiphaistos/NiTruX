<!-- src/pages/SystemToolsPage.vue -->
<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import { systemToolsCatalog, type SystemTool } from "@/data/systemToolsCatalog";

const CATEGORY_LABELS: Record<SystemTool["category"], string> = {
  diagnostics: "Diagnostics",
  reseau: "Réseau",
  performance: "Performance",
  nettoyage: "Nettoyage",
  stockage: "Stockage",
  privilegie: "Privilégié",
};

const activeCategory = ref<string>("all");
const search = ref("");
const running = ref<string | null>(null);
const outputs = ref<Record<string, string>>({});
const errors = ref<Record<string, string>>({});

const categories = computed(() => ["all", ...new Set(systemToolsCatalog.map((t) => t.category))]);

const filteredCatalog = computed<SystemTool[]>(() => {
  let result = systemToolsCatalog;
  if (activeCategory.value !== "all") {
    result = result.filter((t) => t.category === activeCategory.value);
  }
  if (search.value) {
    const q = search.value.toLowerCase();
    result = result.filter((t) => t.name.toLowerCase().includes(q) || t.description.toLowerCase().includes(q));
  }
  return result;
});

async function runTool(tool: SystemTool) {
  running.value = tool.id;
  delete errors.value[tool.id];
  try {
    const output = tool.privilegedAction
      ? await invoke<string>("run_system_tool", { action: tool.privilegedAction })
      : await invoke<string>("run_script", { content: tool.command });
    outputs.value = { ...outputs.value, [tool.id]: output || "(terminé, aucune sortie)" };
  } catch (e) {
    errors.value = { ...errors.value, [tool.id]: String(e) };
  } finally {
    running.value = null;
  }
}
</script>

<template>
  <div class="st-page">
    <NxSectionHeader title="Outils système" description="Commandes courantes en un clic, sans terminal." />

    <div class="st-chips">
      <button
        v-for="cat in categories"
        :key="cat"
        class="st-chip"
        :class="{ active: activeCategory === cat }"
        @click="activeCategory = cat"
      >
        {{ cat === "all" ? "Tout" : CATEGORY_LABELS[cat as SystemTool["category"]] }}
      </button>
    </div>

    <NxInput v-model="search" placeholder="Rechercher un outil..." />

    <div class="st-grid">
      <NxCard v-for="tool in filteredCatalog" :key="tool.id" class="st-card">
        <div class="st-card-header">
          <strong>{{ tool.name }}</strong>
          <NxBadge v-if="tool.privilegedAction" status="warning">root</NxBadge>
        </div>
        <p class="st-desc">{{ tool.description }}</p>
        <NxButton :disabled="running === tool.id" @click="runTool(tool)">
          {{ running === tool.id ? "Exécution..." : "Exécuter" }}
        </NxButton>
        <pre v-if="outputs[tool.id]" class="st-output">{{ outputs[tool.id] }}</pre>
        <NxCard v-if="errors[tool.id]" danger class="st-error">{{ errors[tool.id] }}</NxCard>
      </NxCard>
    </div>
  </div>
</template>

<style scoped>
.st-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.st-chips { display: flex; gap: 8px; flex-wrap: wrap; }
.st-chip { padding: 6px 14px; border-radius: 99px; border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-secondary); cursor: pointer; font: inherit; font-size: 12px; }
.st-chip.active { color: var(--nx-text-primary); font-weight: 600; border-color: var(--nx-accent-primary); }
.st-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 14px; }
.st-card { display: flex; flex-direction: column; gap: 8px; }
.st-card-header { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.st-desc { font-size: 12px; color: var(--nx-text-secondary); margin: 0; }
.st-output { font-size: 11px; background: var(--nx-bg-base); padding: 8px; border-radius: 6px; max-height: 160px; overflow: auto; white-space: pre-wrap; word-break: break-word; }
.st-error { font-size: 12px; padding: 8px 10px; }
</style>
