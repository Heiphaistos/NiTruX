<!-- src/pages/ComingSoonPage.vue -->
<script setup lang="ts">
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

defineProps<{ title: string; phase: string }>();
</script>

<template>
  <div class="cs-page">
    <NxSectionHeader :title="title" />
    <NxCard class="cs-card">
      <p class="cs-message">Bientôt disponible — prévu pour {{ phase }} de la refonte NiTruX.</p>
    </NxCard>
  </div>
</template>

<style scoped>
.cs-page { padding: 24px; }
.cs-card { text-align: center; padding: 40px 20px; }
.cs-message { color: var(--nx-text-secondary); margin: 0; }
</style>
