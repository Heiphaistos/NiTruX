<!-- src/pages/ConfigProfilesPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { useProfilesStore, type ConfigProfile } from "@/stores/profilesStore";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

const profiles = useProfilesStore();
const newProfileName = ref("");
const applied = ref<string | null>(null);
const importError = ref<string | null>(null);
const fileInput = ref<HTMLInputElement | null>(null);

function saveCurrent() {
  const name = newProfileName.value.trim();
  if (name === "") return;
  profiles.saveCurrentAs(name);
  newProfileName.value = "";
}

function applyProfile(profile: ConfigProfile) {
  profiles.apply(profile);
  applied.value = profile.name;
}

function exportProfile(profile: ConfigProfile) {
  const json = profiles.exportProfile(profile);
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `${profile.name.replace(/\s+/g, "_")}.profile.json`;
  a.click();
  URL.revokeObjectURL(url);
}

function handleImportClick() {
  fileInput.value?.click();
}

function handleFileImport(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  importError.value = null;
  const reader = new FileReader();
  reader.onload = (ev) => {
    const result = profiles.importProfile(ev.target?.result as string);
    if (!result.ok) importError.value = result.error;
  };
  reader.readAsText(file);
  (event.target as HTMLInputElement).value = "";
}
</script>

<template>
  <div class="cp-page">
    <NxSectionHeader
      title="Profils de configuration"
      description="Enregistrez le thème, la disposition, le style et les préférences actuels sous un nom, pour basculer rapidement entre plusieurs configurations ou les partager."
    />

    <NxCard class="cp-card">
      <label class="cp-label">Nom du profil</label>
      <div class="cp-save-row">
        <NxInput v-model="newProfileName" placeholder="ex: Bureau, Présentation..." aria-label="Nom du nouveau profil" />
        <NxButton :disabled="newProfileName.trim() === ''" @click="saveCurrent">
          Enregistrer la configuration actuelle
        </NxButton>
      </div>
    </NxCard>

    <NxCard v-if="importError" danger>{{ importError }}</NxCard>
    <NxBadge v-if="applied" status="success" live>Profil « {{ applied }} » appliqué.</NxBadge>

    <NxCard v-if="profiles.profiles.length === 0" class="cp-empty">
      Aucun profil enregistré pour le moment.
    </NxCard>

    <NxCard v-for="profile in profiles.profiles" :key="profile.id" class="cp-profile-row">
      <span class="cp-profile-name">{{ profile.name }}</span>
      <div class="cp-profile-actions">
        <NxButton @click="applyProfile(profile)">Appliquer</NxButton>
        <NxButton @click="exportProfile(profile)">Exporter</NxButton>
        <NxButton variant="danger" @click="profiles.remove(profile.id)">Supprimer</NxButton>
      </div>
    </NxCard>

    <NxButton @click="handleImportClick">Importer un profil</NxButton>
    <input ref="fileInput" type="file" accept=".json" style="display:none" @change="handleFileImport" />
  </div>
</template>

<style scoped>
.cp-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.cp-card { display: flex; flex-direction: column; gap: 8px; }
.cp-label { font-size: 13px; color: var(--nx-text-secondary); }
.cp-save-row { display: flex; gap: 10px; }
.cp-empty { color: var(--nx-text-secondary); }
.cp-profile-row { display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap; }
.cp-profile-name { font-weight: 600; }
.cp-profile-actions { display: flex; gap: 8px; }
</style>
