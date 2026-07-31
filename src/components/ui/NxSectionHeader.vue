<script setup lang="ts">
defineProps<{ title: string; description?: string }>();
</script>

<template>
  <div class="nx-section-header">
    <h2 class="nx-section-header__title">{{ title }}</h2>
    <p v-if="description" class="nx-section-header__description">{{ description }}</p>
  </div>
</template>

<style scoped>
.nx-section-header { margin-bottom: 16px; }
.nx-section-header__title { margin: 0; font-size: 18px; font-weight: 700; color: var(--nx-text-primary); font-family: var(--nx-style-font-family); }
.nx-section-header__description { margin: 4px 0 0; font-size: 13px; color: var(--nx-text-secondary); }
</style>
