<!-- src/components/nav/AppNav.vue -->
<script setup lang="ts">
import { navigationCategories } from "@/navigation/categories";

defineProps<{ modelValue: string }>();
defineEmits<{ "update:modelValue": [string] }>();
</script>

<template>
  <nav class="nx-app-nav">
    <div v-for="category in navigationCategories" :key="category.id" class="nx-app-nav__category">
      <div class="nx-app-nav__title">{{ category.title }}</div>
      <button
        v-for="page in category.pages"
        :key="page.id"
        class="nx-app-nav__item"
        :class="{ active: modelValue === page.id }"
        @click="$emit('update:modelValue', page.id)"
      >
        {{ page.label }}
      </button>
    </div>
  </nav>
</template>

<style scoped>
.nx-app-nav { display: flex; flex-direction: column; gap: 2px; padding: 12px 8px; font-family: var(--nx-style-font-family); }
.nx-app-nav__category { margin-bottom: 10px; }
.nx-app-nav__title {
  padding: 6px 10px 4px;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--nx-text-secondary);
  opacity: 0.7;
  font-weight: 700;
}
.nx-app-nav__item {
  display: block;
  width: 100%;
  text-align: left;
  padding: 7px 10px;
  border: none;
  background: transparent;
  color: var(--nx-text-secondary);
  border-radius: var(--nx-style-radius);
  cursor: pointer;
  font: inherit;
  font-size: 13px;
}
.nx-app-nav__item:hover { background: var(--nx-style-bg); color: var(--nx-text-primary); }
.nx-app-nav__item.active { background: var(--nx-style-bg); color: var(--nx-text-primary); font-weight: 600; }
</style>
