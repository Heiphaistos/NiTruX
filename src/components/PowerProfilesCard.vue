<!-- src/components/PowerProfilesCard.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";

interface PowerProfiles { active: string; available: string[] }

const LABELS: Record<string, string> = {
  "power-saver": "Économie d'énergie",
  balanced: "Équilibré",
  performance: "Performance",
};

const profiles = ref<PowerProfiles | null>(null);
const error = ref<string | null>(null);
const busy = ref(false);

async function load() {
  try {
    profiles.value = await invoke<PowerProfiles>("get_power_profiles");
    error.value = null;
  } catch (e) {
    error.value = String(e);
  }
}

async function choose(profile: string) {
  busy.value = true;
  try {
    await invoke("set_power_profile", { profile });
    await load();
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

onMounted(load);
</script>

<template>
  <NxCard class="pp-card">
    <h3 class="pp-title">Profil d'alimentation</h3>
    <p class="pp-hint">Réglage du système (power-profiles-daemon), réversible à tout moment.</p>
    <div v-if="error" class="pp-error">{{ error }}</div>
    <div v-if="profiles" class="pp-buttons">
      <NxButton
        v-for="p in profiles.available"
        :key="p"
        :disabled="busy || p === profiles.active"
        :aria-pressed="p === profiles.active"
        @click="choose(p)"
      >
        {{ LABELS[p] ?? p }}{{ p === profiles.active ? " (actif)" : "" }}
      </NxButton>
    </div>
  </NxCard>
</template>

<style scoped>
.pp-card { display: flex; flex-direction: column; gap: 8px; }
.pp-title { margin: 0; font-size: 15px; }
.pp-hint { margin: 0; color: var(--nx-text-secondary); font-size: 13px; }
.pp-error { color: var(--nx-accent-danger); font-size: 13px; }
.pp-buttons { display: flex; gap: 8px; flex-wrap: wrap; }
</style>
