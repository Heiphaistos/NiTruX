<!-- src/pages/AiAssistantPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxSelect from "@/components/ui/NxSelect.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import { pendingTerminalCommand } from "@/lib/terminalCommand";

interface OllamaStatus { installed: boolean; running: boolean; models: string[]; error: string | null }
interface ChatMessage { role: "user" | "assistant"; content: string }

const emit = defineEmits<{ navigate: [string] }>();
const status = ref<OllamaStatus | null>(null);
const model = ref("");
const messages = ref<ChatMessage[]>([]);
const draft = ref("");
const thinking = ref(false);
const error = ref<string | null>(null);

async function refresh() {
  status.value = await invoke<OllamaStatus>("ollama_status");
  if (!status.value.models.includes(model.value)) model.value = status.value.models[0] ?? "";
}

onMounted(() => refresh().catch((e) => (error.value = String(e))));

async function send() {
  const text = draft.value.trim();
  if (!text || !model.value || thinking.value) return;
  messages.value.push({ role: "user", content: text });
  draft.value = "";
  thinking.value = true;
  error.value = null;
  try {
    const reply = await invoke<string>("ollama_chat", { model: model.value, messages: messages.value });
    messages.value.push({ role: "assistant", content: reply });
  } catch (e) {
    error.value = String(e);
  } finally {
    thinking.value = false;
  }
}

function inTerminal(command: string) {
  pendingTerminalCommand.value = command;
  emit("navigate", "terminal");
}
</script>

<template>
  <div class="ai-page">
    <NxSectionHeader title="Assistant IA local" description="Un modèle qui tourne sur cette machine avec Ollama : rien ne part sur Internet." />

    <NxCard v-if="status && !status.running" class="ai-setup">
      <template v-if="!status.installed">
        <p>Ollama n'est pas installé.</p>
        <NxButton @click="emit('navigate', 'dependencies')">Installer depuis Dépendances</NxButton>
      </template>
      <template v-else>
        <p>{{ status.error }}</p>
        <NxButton @click="inTerminal('sudo systemctl enable --now ollama')">Démarrer Ollama</NxButton>
      </template>
      <NxButton variant="ghost" @click="refresh">Vérifier à nouveau</NxButton>
    </NxCard>

    <NxCard v-else-if="status && status.models.length === 0" class="ai-setup">
      <p>Ollama tourne, mais aucun modèle n'est téléchargé. Un petit modèle (2 Go) suffit pour l'aide à la maintenance.</p>
      <NxButton @click="inTerminal('ollama pull llama3.2:3b')">Télécharger llama3.2:3b</NxButton>
      <NxButton variant="ghost" @click="refresh">Vérifier à nouveau</NxButton>
    </NxCard>

    <template v-else-if="status">
      <div class="ai-toolbar">
        <NxSelect v-model="model" :options="status.models.map((m) => ({ value: m, label: m }))" aria-label="Modèle" />
        <NxButton variant="ghost" :disabled="messages.length === 0" @click="messages = []">Nouvelle conversation</NxButton>
      </div>
      <NxCard class="ai-chat">
        <p v-if="messages.length === 0" class="ai-empty">Posez une question : « pourquoi mon disque est plein ? », « que fait cette commande ? »…</p>
        <div v-for="(m, i) in messages" :key="i" class="ai-msg" :class="m.role">{{ m.content }}</div>
        <div v-if="thinking" class="ai-msg assistant ai-thinking">Réflexion…</div>
      </NxCard>
      <div v-if="error" class="ai-error">{{ error }}</div>
      <form class="ai-input" @submit.prevent="send">
        <textarea v-model="draft" rows="3" aria-label="Votre question" placeholder="Votre question…" @keydown.enter.exact.prevent="send"></textarea>
        <NxButton type="submit" :disabled="thinking || !draft.trim()">Envoyer</NxButton>
      </form>
    </template>
  </div>
</template>

<style scoped>
.ai-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; height: 100%; box-sizing: border-box; }
.ai-setup { display: flex; flex-direction: column; gap: 8px; align-items: flex-start; }
.ai-setup p { margin: 0; }
.ai-toolbar { display: flex; gap: 8px; align-items: center; }
.ai-chat { flex: 1; min-height: 200px; overflow: auto; display: flex; flex-direction: column; gap: 10px; }
.ai-empty { color: var(--nx-text-secondary); margin: 0; }
.ai-msg { white-space: pre-wrap; padding: 8px 12px; border-radius: var(--nx-style-radius); max-width: 85%; font-size: 13px; }
.ai-msg.user { align-self: flex-end; background: color-mix(in srgb, var(--nx-accent-primary) 20%, transparent); }
.ai-msg.assistant { align-self: flex-start; background: var(--nx-bg-elevated); }
.ai-thinking { color: var(--nx-text-secondary); }
.ai-error { color: var(--nx-accent-danger); font-size: 13px; }
.ai-input { display: flex; gap: 8px; align-items: flex-end; }
.ai-input textarea { flex: 1; resize: vertical; font: inherit; padding: 8px; border-radius: var(--nx-style-radius); border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-primary); }
</style>
