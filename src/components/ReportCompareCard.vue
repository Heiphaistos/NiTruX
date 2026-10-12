<!-- src/components/ReportCompareCard.vue -->
<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxSelect from "@/components/ui/NxSelect.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import { diffReportLines } from "@/lib/reportDiff";

const props = defineProps<{ filenames: string[] }>();

// PDF is binary: only text formats can be compared line by line.
const options = computed(() => props.filenames.filter((f) => !f.endsWith(".pdf")).map((f) => ({ value: f, label: f })));
const before = ref("");
const after = ref("");
watch(options, (opts) => {
  // Reports are listed newest first: default to "previous vs latest".
  if (!opts.some((o) => o.value === after.value)) after.value = opts[0]?.value ?? "";
  if (!opts.some((o) => o.value === before.value)) before.value = opts[1]?.value ?? "";
}, { immediate: true });

const result = ref<{ removed: string[]; added: string[] } | null>(null);
const error = ref<string | null>(null);

async function compare() {
  error.value = null;
  result.value = null;
  try {
    const [a, b] = await Promise.all([
      invoke<string>("read_report", { filename: before.value }),
      invoke<string>("read_report", { filename: after.value }),
    ]);
    result.value = diffReportLines(a, b);
  } catch (e) {
    error.value = String(e);
  }
}
</script>

<template>
  <NxCard v-if="options.length >= 2" class="rc-card">
    <NxSectionHeader title="Comparer deux rapports" description="Ce qui a changé sur la machine entre deux rapports (avant / après)." />
    <div class="rc-row">
      <NxSelect v-model="before" :options="options" aria-label="Rapport avant" />
      <span>→</span>
      <NxSelect v-model="after" :options="options" aria-label="Rapport après" />
      <NxButton :disabled="!before || !after || before === after" @click="compare">Comparer</NxButton>
    </div>
    <div v-if="error" class="rc-error">{{ error }}</div>
    <div v-if="result" class="rc-result">
      <p v-if="!result.removed.length && !result.added.length">Aucune différence.</p>
      <pre v-for="l in result.removed" :key="'-' + l" class="rc-removed">- {{ l }}</pre>
      <pre v-for="l in result.added" :key="'+' + l" class="rc-added">+ {{ l }}</pre>
    </div>
  </NxCard>
</template>

<style scoped>
.rc-card { display: flex; flex-direction: column; gap: 10px; }
.rc-row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.rc-error { color: var(--nx-accent-danger); font-size: 13px; }
.rc-result { max-height: 360px; overflow: auto; font-size: 12px; }
.rc-result pre { margin: 0; white-space: pre-wrap; word-break: break-word; }
.rc-removed { color: var(--nx-accent-danger); }
.rc-added { color: var(--nx-accent-success, var(--nx-accent-primary)); }
</style>
