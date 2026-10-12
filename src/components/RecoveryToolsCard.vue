<!-- src/components/RecoveryToolsCard.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import { pendingTerminalCommand } from "@/lib/terminalCommand";

interface ToolStatus { binary: string; package: string | null; installed: boolean }

const emit = defineEmits<{ navigate: [string] }>();
const testdisk = ref<ToolStatus | null>(null);

onMounted(async () => {
  try {
    const tools = await invoke<ToolStatus[]>("check_required_tools");
    testdisk.value = tools.find((t) => t.binary === "testdisk") ?? null;
  } catch {
    testdisk.value = null;
  }
});

// Both tools read raw disks, so they need root, and both are full-screen
// text interfaces: the integrated terminal runs them under sudo, with the
// user answering every question. Nothing is written to the source disk.
function launch(tool: "photorec" | "testdisk") {
  pendingTerminalCommand.value = `sudo ${tool}`;
  emit("navigate", "terminal");
}
</script>

<template>
  <NxCard class="rt-card">
    <h3 class="rt-title">Fichiers effacés hors corbeille</h3>
    <p class="rt-hint">
      PhotoRec retrouve les fichiers (photos, documents…) sur un disque, une clé ou une carte mémoire, même formatés.
      TestDisk répare une table de partitions ou un secteur de démarrage. Enregistrez toujours les fichiers retrouvés sur un AUTRE disque.
    </p>
    <div v-if="testdisk && !testdisk.installed" class="rt-missing">
      Non installé : paquet « {{ testdisk.package ?? "testdisk" }} ».
      <NxButton @click="emit('navigate', 'dependencies')">Installer</NxButton>
    </div>
    <div v-else class="rt-buttons">
      <NxButton @click="launch('photorec')">Lancer PhotoRec</NxButton>
      <NxButton @click="launch('testdisk')">Lancer TestDisk</NxButton>
    </div>
  </NxCard>
</template>

<style scoped>
.rt-card { display: flex; flex-direction: column; gap: 8px; }
.rt-title { margin: 0; font-size: 15px; }
.rt-hint { margin: 0; color: var(--nx-text-secondary); font-size: 13px; }
.rt-missing { display: flex; align-items: center; gap: 10px; font-size: 13px; }
.rt-buttons { display: flex; gap: 8px; flex-wrap: wrap; }
</style>
