<!-- src/pages/DiskImagePage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSelect from "@/components/ui/NxSelect.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface WholeDisk { path: string; size_bytes: number; model: string; transport: string }
type Mode = "iso" | "restore";

const emit = defineEmits<{ navigate: [string] }>();
const disks = ref<WholeDisk[]>([]);
const error = ref<string | null>(null);
const mode = ref<Mode>("iso");
const image = ref("");
const device = ref("");
const confirmText = ref("");
const busy = ref(false);
const result = ref<string | null>(null);

const gb = (b: number) => `${(b / 1e9).toFixed(1)} Go`;
const label = (d: WholeDisk) => `${d.path} — ${d.model || "disque"} (${gb(d.size_bytes)}${d.transport ? `, ${d.transport}` : ""})`;

// A bootable key can only go to a USB disk (the helper refuses anything
// else); a restore may target any disk but the running system's.
const targets = computed(() => disks.value.filter((d) => mode.value === "restore" || d.transport === "usb"));
const options = computed(() => targets.value.map((d) => ({ value: d.path, label: label(d) })));

async function loadDisks() {
  try {
    disks.value = await invoke<WholeDisk[]>("list_whole_disks");
    error.value = null;
  } catch (e) {
    error.value = String(e);
  }
  if (!targets.value.some((d) => d.path === device.value)) device.value = targets.value[0]?.path ?? "";
}

onMounted(loadDisks);

function setMode(m: Mode) {
  mode.value = m;
  image.value = "";
  confirmText.value = "";
  result.value = null;
  loadDisks();
}

async function pickImage() {
  const picked = await open({
    multiple: false,
    directory: false,
    filters: mode.value === "iso" ? [{ name: "Image ISO", extensions: ["iso", "img"] }] : [{ name: "Image disque", extensions: ["img", "raw", "iso"] }],
  });
  if (typeof picked === "string") image.value = picked;
}

async function write() {
  busy.value = true;
  error.value = null;
  result.value = null;
  try {
    const command = mode.value === "iso" ? "write_iso_to_usb" : "restore_disk_image";
    const args = mode.value === "iso" ? { iso: image.value, device: device.value } : { image: image.value, device: device.value };
    await invoke<string>(command, args);
    result.value = mode.value === "iso" ? `Clé prête : ${device.value} peut démarrer sur l'ISO.` : `Image restaurée sur ${device.value}.`;
    confirmText.value = "";
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="di-page">
    <NxSectionHeader title="Clé USB & image disque" description="Écrire un ISO sur une clé USB (clé de démarrage), ou remettre une image créée avec « Cloner » sur un disque." />

    <div class="di-tabs">
      <NxButton :variant="mode === 'iso' ? 'default' : 'ghost'" @click="setMode('iso')">Clé USB bootable</NxButton>
      <NxButton :variant="mode === 'restore' ? 'default' : 'ghost'" @click="setMode('restore')">Restaurer une image</NxButton>
    </div>

    <NxCard class="di-card">
      <div class="di-row">
        <NxButton @click="pickImage">{{ mode === "iso" ? "Choisir l'ISO…" : "Choisir l'image…" }}</NxButton>
        <code v-if="image">{{ image }}</code>
      </div>
      <p v-if="mode === 'iso'" class="di-hint">
        Vérifiez d'abord l'empreinte de l'ISO téléchargé :
        <a href="#" @click.prevent="emit('navigate', 'file-tools')">Doublons / Gros fichiers / Hash</a>.
      </p>

      <div v-if="targets.length === 0" class="di-hint">
        {{ mode === "iso" ? "Aucune clé USB détectée. Branchez-en une puis actualisez." : "Aucun disque détecté." }}
        <NxButton variant="ghost" @click="loadDisks">Actualiser</NxButton>
      </div>
      <div v-else class="di-row">
        <NxSelect v-model="device" :options="options" aria-label="Disque de destination" />
        <NxButton variant="ghost" @click="loadDisks">Actualiser</NxButton>
      </div>

      <div v-if="image && device" class="di-danger">
        <p>Tout le contenu de <strong>{{ device }}</strong> sera effacé définitivement. Tapez « {{ device }} » pour confirmer.</p>
        <div class="di-row">
          <NxInput v-model="confirmText" :placeholder="device" :aria-label="`Confirmer l'effacement de ${device}`" />
          <NxButton variant="danger" :disabled="busy || confirmText !== device" @click="write">
            {{ busy ? "Écriture en cours… (plusieurs minutes)" : mode === "iso" ? "Écrire sur la clé" : "Restaurer l'image" }}
          </NxButton>
        </div>
      </div>
    </NxCard>

    <NxCard v-if="error" danger>{{ error }}</NxCard>
    <NxCard v-if="result">{{ result }}</NxCard>
  </div>
</template>

<style scoped>
.di-page { padding: 24px; display: flex; flex-direction: column; gap: 14px; }
.di-tabs { display: flex; gap: 8px; }
.di-card { display: flex; flex-direction: column; gap: 12px; }
.di-row { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.di-hint { color: var(--nx-text-secondary); font-size: 13px; margin: 0; }
.di-hint a { color: var(--nx-accent-primary); }
.di-danger { border-left: 3px solid var(--nx-accent-danger); padding-left: 12px; display: flex; flex-direction: column; gap: 8px; }
.di-danger p { margin: 0; font-size: 13px; }
</style>
