<!-- src/pages/BootManagerPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import GrubSettingsCard from "@/components/GrubSettingsCard.vue";
import EfiOrderCard from "@/components/EfiOrderCard.vue";

interface GrubDefaults { default_entry: string | null; timeout_seconds: string | null; distributor: string | null; cmdline_default: string | null }
interface EfiBootEntry { id: string; label: string; is_current: boolean }
interface BootManagerSnapshot { grub: GrubDefaults | null; efi_entries: EfiBootEntry[] | null; efi_boot_order?: string[] | null }

const snapshot = ref<BootManagerSnapshot | null>(null);
const error = ref<string | null>(null);

async function load() {
  try {
    snapshot.value = await invoke<BootManagerSnapshot>("get_boot_manager_snapshot");
  } catch (e) {
    error.value = String(e);
  }
}

onMounted(load);
</script>

<template>
  <div class="bm-page">
    <NxSectionHeader title="Boot Manager" description="Chargeur d'amorçage (GRUB, Limine, systemd-boot) et ordre de démarrage EFI. Chaque changement demande le mot de passe administrateur." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <div v-if="!error && snapshot && !snapshot.grub && !snapshot.efi_entries" class="bm-empty">
      Configuration de démarrage indisponible sur ce système.
    </div>

    <template v-else-if="snapshot">
      <GrubSettingsCard v-if="snapshot.grub" :key="JSON.stringify(snapshot.grub)" :grub="snapshot.grub" @changed="load" />
      <EfiOrderCard
        v-if="snapshot.efi_entries"
        :key="(snapshot.efi_boot_order ?? []).join(',')"
        :entries="snapshot.efi_entries"
        :order="snapshot.efi_boot_order ?? null"
        @changed="load"
      />
    </template>
  </div>
</template>

<style scoped>
.bm-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.bm-empty { color: var(--nx-text-secondary); }
.bm-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
