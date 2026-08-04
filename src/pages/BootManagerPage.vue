<!-- src/pages/BootManagerPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface GrubDefaults { default_entry: string | null; timeout_seconds: string | null; distributor: string | null; cmdline_default: string | null }
interface EfiBootEntry { id: string; label: string; is_current: boolean }
interface BootManagerSnapshot { grub: GrubDefaults | null; efi_entries: EfiBootEntry[] | null }

const snapshot = ref<BootManagerSnapshot | null>(null);

onMounted(async () => {
  snapshot.value = await invoke<BootManagerSnapshot>("get_boot_manager_snapshot");
});
</script>

<template>
  <div class="bm-page">
    <NxSectionHeader title="Boot Manager" description="Configuration GRUB et entrées de démarrage EFI (lecture seule)." />

    <div v-if="snapshot && !snapshot.grub && !snapshot.efi_entries" class="bm-empty">
      Configuration de démarrage indisponible sur ce système.
    </div>

    <template v-else-if="snapshot">
      <NxCard v-if="snapshot.grub">
        <NxSectionHeader title="GRUB" />
        <div class="bm-row"><span>Entrée par défaut</span><span>{{ snapshot.grub.default_entry ?? "—" }}</span></div>
        <div class="bm-row"><span>Délai</span><span>{{ snapshot.grub.timeout_seconds !== null ? `${snapshot.grub.timeout_seconds}s` : "—" }}</span></div>
        <div class="bm-row"><span>Distributeur</span><span>{{ snapshot.grub.distributor ?? "—" }}</span></div>
        <div class="bm-row"><span>Ligne de commande noyau</span><span>{{ snapshot.grub.cmdline_default ?? "—" }}</span></div>
      </NxCard>

      <NxCard v-if="snapshot.efi_entries">
        <NxSectionHeader title="Entrées EFI" />
        <div v-for="e in snapshot.efi_entries" :key="e.id" class="bm-row">
          <span>{{ e.label }} ({{ e.id }})</span>
          <NxBadge :status="e.is_current ? 'success' : 'info'">{{ e.is_current ? "actif" : "inactif" }}</NxBadge>
        </div>
        <div v-if="snapshot.efi_entries.length === 0" class="bm-empty">Aucune entrée de démarrage EFI trouvée.</div>
      </NxCard>
    </template>
  </div>
</template>

<style scoped>
.bm-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.bm-empty { color: var(--nx-text-secondary); }
.bm-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
