<script setup lang="ts">
import { ref } from "vue";
import { useThemeStore, REQUIRED_COLOR_KEYS } from "@/stores/themeStore";
import { useLayoutStore } from "@/stores/layoutStore";
import { builtinThemes } from "@/themes/builtin";
import { layoutRegistry } from "@/layouts/registry";
import type { Theme } from "@/types/theme";

const themeStore = useThemeStore();
const layoutStore = useLayoutStore();
const activeTab = ref<"theme" | "layout">("theme");
const themeName = ref(themeStore.active.name);
const fileInput = ref<HTMLInputElement | null>(null);

const colorKeys = REQUIRED_COLOR_KEYS;

function selectTheme(theme: Theme) {
  themeStore.setTheme(theme);
  themeName.value = theme.name;
}

function handleColorInput(key: keyof Theme["colors"], event: Event) {
  const value = (event.target as HTMLInputElement).value;
  themeStore.updateActiveColor(key, value);
}

function handleSave() {
  themeStore.saveCustomTheme({ ...themeStore.active, id: `custom-${Date.now()}`, name: themeName.value });
}

function handleExport() {
  const json = themeStore.exportActiveTheme();
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `${themeName.value.replace(/\s+/g, "_")}.theme.json`;
  a.click();
  URL.revokeObjectURL(url);
}

function handleImportClick() {
  fileInput.value?.click();
}

function handleFileImport(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = (ev) => {
    const result = themeStore.importTheme(ev.target?.result as string);
    if (!result.ok) alert(result.error);
  };
  reader.readAsText(file);
  (event.target as HTMLInputElement).value = "";
}
</script>

<template>
  <div class="te-page">
    <div class="te-tabs">
      <button :class="{ active: activeTab === 'theme' }" @click="activeTab = 'theme'">Thème</button>
      <button :class="{ active: activeTab === 'layout' }" @click="activeTab = 'layout'">Disposition</button>
    </div>

    <section v-if="activeTab === 'theme'" class="te-panel">
      <input v-model="themeName" class="te-name-input" placeholder="Nom du thème..." />

      <div class="te-swatches">
        <button
          v-for="theme in builtinThemes"
          :key="theme.id"
          class="te-swatch"
          :style="{ background: theme.colors.bgBase, borderColor: theme.colors.accentPrimary }"
          :title="theme.name"
          @click="selectTheme(theme)"
        />
      </div>

      <div class="te-colors">
        <label v-for="key in colorKeys" :key="key" class="te-color-row">
          <span>{{ key }}</span>
          <input type="color" :value="themeStore.active.colors[key]" @input="handleColorInput(key, $event)" />
        </label>
      </div>

      <div class="te-actions">
        <button @click="handleSave">Sauvegarder</button>
        <button @click="handleExport">Exporter</button>
        <button @click="handleImportClick">Importer</button>
        <input ref="fileInput" type="file" accept=".json" style="display:none" @change="handleFileImport" />
      </div>
    </section>

    <section v-else class="te-panel">
      <div class="te-layouts">
        <button
          v-for="layout in layoutRegistry"
          :key="layout.id"
          class="te-layout-option"
          :class="{ active: layoutStore.current === layout.id }"
          @click="layoutStore.setLayout(layout.id)"
        >
          <strong>{{ layout.name }}</strong>
          <p>{{ layout.description }}</p>
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.te-page { padding: 24px; color: var(--nx-text-primary); }
.te-tabs { display: flex; gap: 8px; margin-bottom: 16px; }
.te-tabs button { padding: 8px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-secondary); cursor: pointer; }
.te-tabs button.active { color: var(--nx-text-primary); border-color: var(--nx-accent-primary); }
.te-name-input { width: 100%; padding: 8px 12px; margin-bottom: 16px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); }
.te-swatches { display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 20px; }
.te-swatch { width: 32px; height: 32px; border-radius: 8px; border: 2px solid; cursor: pointer; }
.te-colors { display: grid; gap: 8px; margin-bottom: 20px; }
.te-color-row { display: flex; justify-content: space-between; align-items: center; font-size: 13px; color: var(--nx-text-secondary); }
.te-actions { display: flex; gap: 8px; }
.te-actions button { padding: 8px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); cursor: pointer; }
.te-layouts { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 12px; }
.te-layout-option { text-align: left; padding: 14px; border-radius: 10px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); cursor: pointer; }
.te-layout-option.active { border-color: var(--nx-accent-primary); }
.te-layout-option p { margin: 6px 0 0; font-size: 12px; color: var(--nx-text-secondary); }
</style>
