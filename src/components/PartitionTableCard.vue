<!-- src/components/PartitionTableCard.vue -->
<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSelect from "@/components/ui/NxSelect.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

type Action = "create" | "shrink" | "delete";
const emit = defineEmits<{ changed: [] }>();

const action = ref<Action>("create");
const target = ref("");
const size = ref("");
const confirm = ref("");
const busy = ref(false);
const result = ref<string | null>(null);
const error = ref<string | null>(null);

const ACTIONS = [
  { value: "create", label: "Créer une partition dans l'espace libre" },
  { value: "shrink", label: "Réduire une partition ext4" },
  { value: "delete", label: "Supprimer une partition" },
];

const placeholder = computed(() => (action.value === "create" ? "Disque (ex : /dev/sdb)" : "Partition (ex : /dev/sdb2)"));
const sizeOk = computed(() => /^\d+$/.test(size.value) && (action.value === "create" || Number(size.value) > 0));
// Deleting or shrinking touches existing data: the device must be typed again.
const ready = computed(
  () => target.value !== "" && (action.value === "delete" || sizeOk.value) && (action.value === "create" || confirm.value === target.value),
);

async function run() {
  busy.value = true;
  error.value = null;
  result.value = null;
  try {
    if (action.value === "create") result.value = await invoke<string>("create_partition", { disk: target.value, sizeMib: Number(size.value) });
    else if (action.value === "shrink") result.value = await invoke<string>("shrink_partition", { device: target.value, sizeMib: Number(size.value) });
    else result.value = await invoke<string>("delete_partition", { device: target.value });
    confirm.value = "";
    emit("changed");
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <NxCard staticDanger class="pt-card">
    <NxSectionHeader
      title="Partitions"
      description="Créer, réduire ou supprimer une partition. La partition doit être démontée ; celle du système en cours est toujours refusée. Faites une sauvegarde avant."
    />
    <div class="pt-row">
      <NxSelect v-model="action" :options="ACTIONS" aria-label="Opération sur les partitions" />
      <NxInput v-model="target" :placeholder="placeholder" aria-label="Disque ou partition visé" />
      <NxInput
        v-if="action !== 'delete'"
        v-model="size"
        :placeholder="action === 'create' ? 'Taille en Mio (0 = tout l\'espace libre)' : 'Nouvelle taille en Mio'"
        aria-label="Taille en Mio"
      />
    </div>
    <div class="pt-row">
      <NxInput v-if="action !== 'create'" v-model="confirm" :placeholder="`Tapez « ${target} » pour confirmer`" aria-label="Confirmation de la partition" />
      <NxButton :variant="action === 'create' ? 'default' : 'danger'" :disabled="busy || !ready" @click="run">
        {{ busy ? "En cours…" : action === "create" ? "Créer" : action === "shrink" ? "Réduire" : "Supprimer" }}
      </NxButton>
    </div>
    <div v-if="error" class="pt-error">{{ error }}</div>
    <pre v-if="result" class="pt-result">{{ result }}</pre>
  </NxCard>
</template>

<style scoped>
.pt-card { display: flex; flex-direction: column; gap: 8px; }
.pt-row { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.pt-error { color: var(--nx-accent-danger); font-size: 13px; white-space: pre-wrap; }
.pt-result { margin: 0; font-size: 12px; white-space: pre-wrap; }
</style>
