<!-- src/components/GrubSettingsCard.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface GrubDefaults { default_entry: string | null; timeout_seconds: string | null; distributor: string | null; cmdline_default: string | null }
interface GrubEntry { index: number; title: string }

const props = defineProps<{ grub: GrubDefaults }>();
const emit = defineEmits<{ changed: [] }>();

const timeout = ref(props.grub.timeout_seconds ?? "5");
const entries = ref<GrubEntry[] | null>(null);
const busy = ref(false);
const message = ref<string | null>(null);
const error = ref<string | null>(null);

// Every change asks for the admin password once, then regenerates grub.cfg.
async function run(command: string, args: Record<string, unknown>, done: string) {
  busy.value = true;
  error.value = null;
  message.value = null;
  try {
    await invoke(command, args);
    message.value = done;
    emit("changed");
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function loadEntries() {
  try {
    entries.value = await invoke<GrubEntry[]>("get_grub_entries");
  } catch (e) {
    error.value = String(e);
  }
}

const saved = () => props.grub.default_entry === "saved";
</script>

<template>
  <NxCard class="gs-card">
    <NxSectionHeader title="GRUB" />
    <div class="gs-row"><span>Entrée par défaut</span><span>{{ grub.default_entry === "saved" ? "la dernière choisie" : grub.default_entry ?? "—" }}</span></div>
    <div class="gs-row"><span>Distributeur</span><span>{{ grub.distributor ?? "—" }}</span></div>
    <div class="gs-row"><span>Ligne de commande noyau</span><span>{{ grub.cmdline_default ?? "—" }}</span></div>

    <div class="gs-line">
      <label for="gs-timeout">Délai du menu (secondes)</label>
      <NxInput id="gs-timeout" v-model="timeout" aria-label="Délai du menu GRUB en secondes" />
      <NxButton :disabled="busy || !/^\d+$/.test(timeout) || Number(timeout) > 60" @click="run('set_grub_timeout', { seconds: Number(timeout) }, `Délai réglé à ${timeout} s.`)">Appliquer</NxButton>
    </div>

    <div class="gs-line">
      <NxButton variant="ghost" :disabled="busy" @click="loadEntries">Afficher les entrées du menu (mot de passe)</NxButton>
      <NxButton v-if="!saved()" variant="ghost" :disabled="busy" @click="run('set_grub_default', { entry: 'saved' }, 'GRUB démarrera sur la dernière entrée choisie.')">Retenir le dernier choix</NxButton>
    </div>

    <div v-for="e in entries ?? []" :key="e.index" class="gs-entry">
      <span>{{ e.index }} — {{ e.title }}</span>
      <span class="gs-actions">
        <NxButton variant="ghost" :disabled="busy" @click="run('set_grub_default', { entry: String(e.index) }, `Entrée ${e.index} par défaut.`)">Par défaut</NxButton>
        <NxButton variant="ghost" :disabled="busy || !saved()" :title="saved() ? '' : 'Choisissez d\'abord « Retenir le dernier choix »'" @click="run('grub_reboot_once', { index: e.index }, `Au prochain démarrage seulement : entrée ${e.index}.`)">Prochain démarrage</NxButton>
      </span>
    </div>

    <div v-if="message" class="gs-ok">{{ message }}</div>
    <div v-if="error" class="gs-error">{{ error }}</div>
  </NxCard>
</template>

<style scoped>
.gs-card { display: flex; flex-direction: column; gap: 8px; }
.gs-row, .gs-entry { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
.gs-line { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; font-size: 13px; }
.gs-actions { display: flex; gap: 6px; }
.gs-ok { color: var(--nx-accent-success); font-size: 13px; }
.gs-error { color: var(--nx-accent-danger); font-size: 13px; }
</style>
