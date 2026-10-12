<!-- src/components/EfiOrderCard.vue -->
<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface EfiBootEntry { id: string; label: string; is_current: boolean }

const props = defineProps<{ entries: EfiBootEntry[]; order: string[] | null }>();
const emit = defineEmits<{ changed: [] }>();

// The boot order as shown, entries outside BootOrder appended after it.
const initial = () => [
  ...(props.order ?? []).filter((id) => props.entries.some((e) => e.id === id)),
  ...props.entries.map((e) => e.id).filter((id) => !(props.order ?? []).includes(id)),
];
const draft = ref<string[]>(initial());
const changed = computed(() => props.order !== null && draft.value.join(",") !== props.order.join(","));
const busy = ref(false);
const error = ref<string | null>(null);
const label = (id: string) => props.entries.find((e) => e.id === id)?.label ?? id;
const entry = (id: string) => props.entries.find((e) => e.id === id);

function move(i: number, delta: number) {
  const j = i + delta;
  if (j < 0 || j >= draft.value.length) return;
  const next = [...draft.value];
  [next[i], next[j]] = [next[j]!, next[i]!];
  draft.value = next;
}

async function apply() {
  busy.value = true;
  error.value = null;
  try {
    await invoke("set_efi_boot_order", { order: draft.value });
    emit("changed");
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <NxCard class="eo-card">
    <NxSectionHeader title="Ordre de démarrage EFI" description="Le premier de la liste démarre. Après une mise à jour Windows, remettez Linux en tête." />
    <div v-for="(id, i) in draft" :key="id" class="eo-row">
      <span>{{ i + 1 }}. {{ label(id) }} ({{ id }})</span>
      <span class="eo-actions">
        <NxBadge :status="entry(id)?.is_current ? 'success' : 'info'">{{ entry(id)?.is_current ? "actif" : "inactif" }}</NxBadge>
        <NxButton variant="ghost" :disabled="i === 0" :aria-label="`Monter ${label(id)}`" @click="move(i, -1)">↑</NxButton>
        <NxButton variant="ghost" :disabled="i === draft.length - 1" :aria-label="`Descendre ${label(id)}`" @click="move(i, 1)">↓</NxButton>
      </span>
    </div>
    <div v-if="draft.length === 0" class="eo-empty">Aucune entrée de démarrage EFI trouvée.</div>
    <div v-if="changed" class="eo-line">
      <NxButton :disabled="busy" @click="apply">Appliquer cet ordre</NxButton>
      <NxButton variant="ghost" :disabled="busy" @click="draft = initial()">Annuler</NxButton>
    </div>
    <div v-if="error" class="eo-error">{{ error }}</div>
  </NxCard>
</template>

<style scoped>
.eo-card { display: flex; flex-direction: column; gap: 6px; }
.eo-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
.eo-actions { display: flex; gap: 6px; align-items: center; }
.eo-line { display: flex; gap: 8px; }
.eo-empty { color: var(--nx-text-secondary); }
.eo-error { color: var(--nx-accent-danger); font-size: 13px; }
</style>
