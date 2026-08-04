<!-- src/pages/ScriptsPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import { useScriptsStore } from "@/stores/scriptsStore";

const store = useScriptsStore();

const newName = ref("");
const newContent = ref("");
const saveError = ref<string | null>(null);

function saveScript() {
  if (newName.value === "" || newContent.value === "") return;
  const result = store.addScript(newName.value, newContent.value);
  if (!result.ok) {
    saveError.value = result.error;
    return;
  }
  saveError.value = null;
  newName.value = "";
  newContent.value = "";
}

const running = ref<string | null>(null);
const outputs = ref<Record<string, string>>({});
const errors = ref<Record<string, string>>({});

async function runScript(name: string, content: string) {
  running.value = name;
  delete errors.value[name];
  try {
    outputs.value[name] = await invoke<string>("run_script", { content });
  } catch (e) {
    errors.value[name] = String(e);
  } finally {
    running.value = null;
  }
}
</script>

<template>
  <div class="scr-page">
    <NxSectionHeader title="Scripts & Snippets" description="Enregistre et exécute des scripts shell avec vos propres privilèges — aucune élévation." />

    <NxCard class="scr-new">
      <NxInput v-model="newName" placeholder="Nom du script..." />
      <textarea v-model="newContent" class="scr-textarea" rows="4" placeholder="Contenu du script..."></textarea>
      <NxButton @click="saveScript">Enregistrer</NxButton>
      <NxCard v-if="saveError" danger>{{ saveError }}</NxCard>
    </NxCard>

    <NxCard v-for="s in store.scripts" :key="s.name" class="scr-item">
      <div class="scr-item-header">
        <span>{{ s.name }}</span>
        <div class="scr-item-actions">
          <NxButton :disabled="running !== null" @click="runScript(s.name, s.content)">
            {{ running === s.name ? "En cours..." : "Exécuter" }}
          </NxButton>
          <NxButton variant="danger" @click="store.removeScript(s.name)">Supprimer</NxButton>
        </div>
      </div>
      <pre class="scr-content">{{ s.content }}</pre>
      <NxCard v-if="errors[s.name]" danger>{{ errors[s.name] }}</NxCard>
      <pre v-if="outputs[s.name]" class="scr-output">{{ outputs[s.name] }}</pre>
    </NxCard>
    <NxCard v-if="store.scripts.length === 0" class="scr-empty">Aucun script enregistré pour le moment.</NxCard>
  </div>
</template>

<style scoped>
.scr-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.scr-new { display: flex; flex-direction: column; gap: 10px; }
.scr-textarea { width: 100%; padding: 10px; border-radius: var(--nx-style-radius); border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-primary); font-family: monospace; font-size: 12px; }
.scr-item { display: flex; flex-direction: column; gap: 8px; }
.scr-item-header { display: flex; justify-content: space-between; align-items: center; }
.scr-item-actions { display: flex; gap: 8px; }
.scr-content, .scr-output { font-size: 12px; margin: 0; white-space: pre-wrap; word-break: break-word; }
.scr-output { color: var(--nx-accent-success); }
.scr-empty { color: var(--nx-text-secondary); font-size: 13px; }
</style>
