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

// Quarantine moves the file out of its original location, and this app
// offers no "restore from quarantine" path back -- from the user's
// perspective it is effectively as irreversible as a permanent delete,
// and a scanner false positive on a real, needed file is a genuine risk.
// Mirrors the type-to-confirm pattern already established elsewhere in
// this app for comparably irreversible actions (UninstallerPage's
// uninstall, DisksPage's format-partition, DataRecoveryPage's permanent
// delete) rather than a single unconfirmed click.
const confirmingQuarantine = ref<string | null>(null);
const confirmQuarantineText = ref("");

function startConfirmQuarantine(path: string) {
  confirmingQuarantine.value = path;
  confirmQuarantineText.value = "";
}

async function quarantineFinding(path: string) {
  quarantining.value = path;
  quarantineError.value = null;
  try {
    await invoke("quarantine_file", { path });
    findings.value = findings.value.filter((f) => f.path !== path);
    confirmingQuarantine.value = null;
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
      <div v-for="f in findings" :key="f.path" class="av-finding">
        <div class="av-finding-row">
          <span>{{ f.path }}</span>
          <span>{{ f.signature }}</span>
          <NxButton v-if="confirmingQuarantine !== f.path" variant="danger" :disabled="quarantining !== null" @click="startConfirmQuarantine(f.path)">
            Mettre en quarantaine
          </NxButton>
        </div>
        <div v-if="confirmingQuarantine === f.path" class="av-confirm-row">
          <NxInput
            v-model="confirmQuarantineText"
            :placeholder="`Tapez « ${f.path} » pour confirmer`"
            aria-label="Confirmation du chemin du fichier à mettre en quarantaine"
          />
          <NxButton
            variant="danger"
            :disabled="quarantining !== null || confirmQuarantineText !== f.path"
            @click="quarantineFinding(f.path)"
          >
            {{ quarantining === f.path ? "Mise en quarantaine..." : "Confirmer la mise en quarantaine" }}
          </NxButton>
        </div>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.av-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.av-form-row { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; }
.av-empty { color: var(--nx-text-secondary); }
.av-finding { display: flex; flex-direction: column; gap: 8px; padding: 6px 0; border-bottom: 1px solid var(--nx-style-border-color); }
.av-finding-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; font-size: 13px; }
.av-confirm-row { display: flex; gap: 10px; align-items: center; }
</style>
