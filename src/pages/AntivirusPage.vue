<!-- src/pages/AntivirusPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import { usePreferencesStore } from "@/stores/preferencesStore";

interface MalwareFinding { path: string; signature: string }

const preferences = usePreferencesStore();
const scanDir = ref(preferences.defaultScanDirectory);
const findings = ref<MalwareFinding[]>([]);
const scanError = ref<string | null>(null);
const scanning = ref(false);
const scanDone = ref(false);

async function runScan() {
  scanning.value = true;
  scanError.value = null;
  scanDone.value = false;
  try {
    findings.value = await invoke<MalwareFinding[]>("scan_for_malware", { directory: scanDir.value });
    scanDone.value = true;
  } catch (e) {
    scanError.value = String(e);
  } finally {
    scanning.value = false;
  }
}

const quarantining = ref<string | null>(null);
const quarantineError = ref<string | null>(null);

async function quarantineFinding(path: string) {
  quarantining.value = path;
  quarantineError.value = null;
  try {
    await invoke("quarantine_file", { path });
    findings.value = findings.value.filter((f) => f.path !== path);
  } catch (e) {
    quarantineError.value = String(e);
  } finally {
    quarantining.value = null;
  }
}
</script>

<template>
  <div class="av-page">
    <NxSectionHeader title="Antivirus" description="Analyse un dossier à la recherche de signatures de malware connues et met en quarantaine ce qui est trouvé." />

    <NxCard>
      <div class="av-form-row">
        <NxInput v-model="scanDir" placeholder="Dossier à scanner..." aria-label="Dossier à scanner" />
        <NxButton :disabled="scanning" @click="runScan">{{ scanning ? "Scan en cours..." : "Scanner" }}</NxButton>
      </div>
      <NxCard v-if="scanError" danger>{{ scanError }}</NxCard>
      <div v-else-if="scanDone && findings.length === 0" class="av-empty">Aucune menace détectée.</div>
      <NxCard v-if="quarantineError" danger>{{ quarantineError }}</NxCard>
      <div v-for="f in findings" :key="f.path" class="av-finding-row">
        <span>{{ f.path }}</span>
        <span>{{ f.signature }}</span>
        <NxButton variant="danger" :disabled="quarantining !== null" @click="quarantineFinding(f.path)">
          {{ quarantining === f.path ? "Mise en quarantaine..." : "Mettre en quarantaine" }}
        </NxButton>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.av-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.av-form-row { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; }
.av-empty { color: var(--nx-text-secondary); }
.av-finding-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
