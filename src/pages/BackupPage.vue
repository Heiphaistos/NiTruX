<!-- src/pages/BackupPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

const sourceDir = ref("");
const creating = ref(false);
const error = ref<string | null>(null);
const resultPath = ref<string | null>(null);

async function createBackup() {
  creating.value = true;
  error.value = null;
  resultPath.value = null;
  try {
    resultPath.value = await invoke<string>("create_backup", { sourceDir: sourceDir.value });
  } catch (e) {
    error.value = String(e);
  } finally {
    creating.value = false;
  }
}
</script>

<template>
  <div class="bkp-page">
    <NxSectionHeader title="Sauvegarde" description="Archive un dossier vers un fichier .tar.gz horodaté dans votre dossier personnel." />

    <NxCard>
      <div class="bkp-form-row">
        <NxInput v-model="sourceDir" placeholder="Dossier à sauvegarder (ex: /home/dev/documents)" />
        <NxButton :disabled="creating || sourceDir === ''" @click="createBackup">{{ creating ? "Sauvegarde en cours..." : "Créer la sauvegarde" }}</NxButton>
      </div>
      <NxCard v-if="error" danger>{{ error }}</NxCard>
      <NxBadge v-if="resultPath" status="success">Sauvegarde créée : {{ resultPath }}</NxBadge>
    </NxCard>
  </div>
</template>

<style scoped>
.bkp-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.bkp-form-row { display: flex; gap: 10px; align-items: center; }
</style>
