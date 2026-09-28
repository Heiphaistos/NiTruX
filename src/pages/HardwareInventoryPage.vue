<!-- src/pages/HardwareInventoryPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface KeyValue { key: string; value: string }
interface Section { title: string; items: KeyValue[] }
interface Device { name: string; details: KeyValue[] }
interface Inventory {
  cpu: Section;
  cpu_cores: KeyValue[];
  firmware: Section;
  memory: Section;
  gpus: Device[];
  storage: Device[];
  power: Device[];
  network: Device[];
  usb: Device[];
  audio: Device[];
}
interface MemoryModules { max_capacity: string | null; slots_total: number | null; modules: Device[]; empty_slots: string[] }
interface SmartReport { device: string; healthy: boolean | null; attributes: KeyValue[] }

const inv = ref<Inventory | null>(null);
const error = ref<string | null>(null);

// Admin-only extras, each behind its own button + one polkit prompt.
const modules = ref<MemoryModules | null>(null);
const modulesError = ref<string | null>(null);
const modulesBusy = ref(false);
const winKey = ref<string | null>(null);
const winKeyChecked = ref(false);
const winKeyBusy = ref(false);
const smart = ref<Record<string, SmartReport>>({});
const smartBusy = ref<string | null>(null);

async function load() {
  error.value = null;
  try {
    inv.value = await invoke<Inventory>("get_hardware_inventory");
  } catch (e) {
    error.value = String(e);
  }
}

async function loadModules() {
  modulesBusy.value = true;
  modulesError.value = null;
  try {
    modules.value = await invoke<MemoryModules>("get_memory_modules");
  } catch (e) {
    modulesError.value = String(e);
  } finally {
    modulesBusy.value = false;
  }
}

async function checkWindowsKey() {
  winKeyBusy.value = true;
  try {
    winKey.value = await invoke<string | null>("get_firmware_windows_key");
    winKeyChecked.value = true;
  } catch (e) {
    winKey.value = null;
    error.value = String(e);
  } finally {
    winKeyBusy.value = false;
  }
}

function deviceNode(dev: Device): string {
  return dev.details.find((d) => d.key === "Périphérique")?.value ?? "";
}

async function loadSmart(dev: Device) {
  const node = deviceNode(dev);
  if (!node) return;
  smartBusy.value = node;
  try {
    smart.value = { ...smart.value, [node]: await invoke<SmartReport>("get_smart_report", { device: node }) };
  } catch (e) {
    smart.value = { ...smart.value, [node]: { device: node, healthy: null, attributes: [{ key: "Erreur", value: String(e) }] } };
  } finally {
    smartBusy.value = null;
  }
}

onMounted(load);
</script>

<template>
  <div class="hw-page">
    <NxSectionHeader title="Matériel complet" description="Inventaire détaillé lu directement depuis le noyau — processeur, firmware, mémoire, GPU, stockage, alimentation, réseau, USB, audio." />
    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <template v-if="inv">
      <!-- CPU -->
      <NxCard class="hw-section">
        <h3>{{ inv.cpu.title }}</h3>
        <div class="hw-kv-grid">
          <div v-for="i in inv.cpu.items" :key="i.key" class="hw-kv"><span>{{ i.key }}</span><strong>{{ i.value }}</strong></div>
        </div>
        <details v-if="inv.cpu_cores.length">
          <summary>Fréquence par cœur ({{ inv.cpu_cores.length }})</summary>
          <div class="hw-cores">
            <span v-for="c in inv.cpu_cores" :key="c.key" class="hw-core">{{ c.key }} · {{ c.value }}</span>
          </div>
        </details>
      </NxCard>

      <!-- Firmware / motherboard, with admin extras -->
      <NxCard class="hw-section">
        <h3>{{ inv.firmware.title }}</h3>
        <div class="hw-kv-grid">
          <div v-for="i in inv.firmware.items" :key="i.key" class="hw-kv"><span>{{ i.key }}</span><strong>{{ i.value }}</strong></div>
        </div>
        <div class="hw-actions">
          <NxButton :disabled="winKeyBusy" @click="checkWindowsKey">
            {{ winKeyBusy ? "Lecture..." : "Clé Windows OEM du firmware (admin)" }}
          </NxButton>
          <NxBadge v-if="winKeyChecked && winKey" status="info" live><code>{{ winKey }}</code></NxBadge>
          <NxBadge v-else-if="winKeyChecked" status="warning" live>Aucune clé OEM dans le firmware.</NxBadge>
        </div>
      </NxCard>

      <!-- Memory -->
      <NxCard class="hw-section">
        <h3>{{ inv.memory.title }}</h3>
        <div class="hw-kv-grid">
          <div v-for="i in inv.memory.items" :key="i.key" class="hw-kv"><span>{{ i.key }}</span><strong>{{ i.value }}</strong></div>
        </div>
        <div class="hw-actions">
          <NxButton :disabled="modulesBusy" @click="loadModules">
            {{ modulesBusy ? "Lecture..." : "Détail des barrettes (admin)" }}
          </NxButton>
        </div>
        <NxCard v-if="modulesError" danger>{{ modulesError }}</NxCard>
        <template v-if="modules">
          <p class="hw-muted">
            {{ modules.modules.length }} barrette(s) installée(s)<template v-if="modules.slots_total"> sur {{ modules.slots_total }} emplacement(s)</template><template v-if="modules.max_capacity"> — max {{ modules.max_capacity }}</template>.
          </p>
          <div class="hw-devices">
            <NxCard v-for="(m, i) in modules.modules" :key="i" class="hw-device">
              <strong>{{ m.name }}</strong>
              <div v-for="d in m.details" :key="d.key" class="hw-kv"><span>{{ d.key }}</span><strong>{{ d.value }}</strong></div>
            </NxCard>
          </div>
          <p v-if="modules.empty_slots.length" class="hw-muted">Emplacements libres : {{ modules.empty_slots.join(", ") }}</p>
        </template>
      </NxCard>

      <!-- Device groups -->
      <NxCard v-for="group in [
        { title: 'Cartes graphiques', list: inv.gpus },
        { title: 'Stockage', list: inv.storage },
        { title: 'Alimentation & batterie', list: inv.power },
        { title: 'Réseau', list: inv.network },
        { title: 'Périphériques USB', list: inv.usb },
        { title: 'Audio', list: inv.audio },
      ]" :key="group.title" class="hw-section">
        <h3>{{ group.title }} <span class="hw-count">{{ group.list.length }}</span></h3>
        <p v-if="group.list.length === 0" class="hw-muted">Aucun détecté.</p>
        <div v-else class="hw-devices">
          <NxCard v-for="(dev, i) in group.list" :key="i" class="hw-device">
            <div class="hw-device-head">
              <strong>{{ dev.name }}</strong>
              <NxButton
                v-if="group.title === 'Stockage' && deviceNode(dev)"
                variant="ghost"
                :disabled="smartBusy === deviceNode(dev)"
                @click="loadSmart(dev)"
              >{{ smartBusy === deviceNode(dev) ? "S.M.A.R.T…" : "Santé S.M.A.R.T. (admin)" }}</NxButton>
            </div>
            <div v-for="d in dev.details" :key="d.key" class="hw-kv"><span>{{ d.key }}</span><strong>{{ d.value }}</strong></div>
            <template v-if="smart[deviceNode(dev)]">
              <NxBadge :status="smart[deviceNode(dev)].healthy === false ? 'danger' : smart[deviceNode(dev)].healthy ? 'success' : 'warning'">
                {{ smart[deviceNode(dev)].healthy === false ? "DÉFAILLANT" : smart[deviceNode(dev)].healthy ? "Sain" : "État inconnu" }}
              </NxBadge>
              <div v-for="a in smart[deviceNode(dev)].attributes" :key="a.key" class="hw-kv"><span>{{ a.key }}</span><strong>{{ a.value }}</strong></div>
            </template>
          </NxCard>
        </div>
      </NxCard>
    </template>
    <p v-else-if="!error" class="hw-muted">Lecture du matériel…</p>
  </div>
</template>

<style scoped>
.hw-page { padding: 24px; display: flex; flex-direction: column; gap: 14px; }
.hw-section { display: flex; flex-direction: column; gap: 10px; }
.hw-section h3 { margin: 0; font-size: 15px; display: flex; align-items: center; gap: 8px; }
.hw-count { font-size: 12px; color: var(--nx-text-secondary); font-weight: 400; }
.hw-kv-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 4px 20px; }
.hw-kv { display: flex; justify-content: space-between; gap: 12px; font-size: 13px; padding: 3px 0; border-bottom: 1px solid var(--nx-style-border-color); }
.hw-kv span { color: var(--nx-text-secondary); }
.hw-kv strong { text-align: right; word-break: break-word; }
.hw-devices { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 12px; }
.hw-device { display: flex; flex-direction: column; gap: 4px; }
.hw-device-head { display: flex; justify-content: space-between; align-items: center; gap: 8px; }
.hw-actions { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
.hw-cores { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 8px; }
.hw-core { font-size: 11px; padding: 2px 8px; border-radius: 99px; background: var(--nx-bg-elevated); color: var(--nx-text-secondary); }
.hw-muted { font-size: 12px; color: var(--nx-text-secondary); margin: 2px 0; }
details summary { cursor: pointer; font-size: 13px; color: var(--nx-text-secondary); }
</style>
