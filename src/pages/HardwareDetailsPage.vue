<!-- src/pages/HardwareDetailsPage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface CpuDetails { model_name: string | null; sockets: string | null; cores_per_socket: string | null; threads_per_core: string | null }
interface BoardDetails { product_name: string | null; sys_vendor: string | null; board_name: string | null; bios_version: string | null }
interface MemoryDetails { total_kb: number | null; available_kb: number | null; cached_kb: number | null; swap_total_kb: number | null }
interface HardwareDetails { cpu: CpuDetails; board: BoardDetails; memory: MemoryDetails }
interface PciDevice { slot: string; class: string; description: string }

const details = ref<HardwareDetails | null>(null);
const gpus = ref<PciDevice[]>([]);

onMounted(async () => {
  details.value = await invoke<HardwareDetails>("get_hardware_details");
  const pci = await invoke<PciDevice[]>("get_pci_devices");
  gpus.value = pci.filter((d) => d.class.includes("VGA") || d.class.includes("3D"));
});

function kbToGb(kb: number | null): string {
  return kb === null ? "—" : (kb / 1024 / 1024).toFixed(1);
}

const memoryRows = computed(() => details.value ? [
  { label: "Total", value: kbToGb(details.value.memory.total_kb) + " GB" },
  { label: "Disponible", value: kbToGb(details.value.memory.available_kb) + " GB" },
  { label: "Cache", value: kbToGb(details.value.memory.cached_kb) + " GB" },
  { label: "Swap total", value: kbToGb(details.value.memory.swap_total_kb) + " GB" },
] : []);
</script>

<template>
  <div class="hd-page">
    <NxSectionHeader title="Matériel détaillé" description="Processeur, carte mère, mémoire et GPU." />

    <template v-if="details">
      <NxCard>
        <NxSectionHeader title="Processeur" />
        <div class="hd-row"><span>Modèle</span><span>{{ details.cpu.model_name ?? "—" }}</span></div>
        <div class="hd-row"><span>Socket(s)</span><span>{{ details.cpu.sockets ?? "—" }}</span></div>
        <div class="hd-row"><span>Cœurs par socket</span><span>{{ details.cpu.cores_per_socket ?? "—" }}</span></div>
        <div class="hd-row"><span>Threads par cœur</span><span>{{ details.cpu.threads_per_core ?? "—" }}</span></div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Carte mère & BIOS" />
        <div class="hd-row"><span>Fabricant</span><span>{{ details.board.sys_vendor ?? "—" }}</span></div>
        <div class="hd-row"><span>Modèle</span><span>{{ details.board.product_name ?? "—" }}</span></div>
        <div class="hd-row"><span>Carte mère</span><span>{{ details.board.board_name ?? "—" }}</span></div>
        <div class="hd-row"><span>Version BIOS</span><span>{{ details.board.bios_version ?? "—" }}</span></div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Mémoire" />
        <div v-for="row in memoryRows" :key="row.label" class="hd-row">
          <span>{{ row.label }}</span><span>{{ row.value }}</span>
        </div>
      </NxCard>

      <NxCard v-if="gpus.length">
        <NxSectionHeader title="GPU" />
        <div v-for="g in gpus" :key="g.slot" class="hd-row">
          <span>{{ g.description }}</span><span>{{ g.slot }}</span>
        </div>
      </NxCard>
    </template>
  </div>
</template>

<style scoped>
.hd-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.hd-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
