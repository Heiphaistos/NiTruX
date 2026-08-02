<!-- src/pages/UserAccountsPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface UserAccount { username: string; uid: number; home: string; shell: string }

const accounts = ref<UserAccount[]>([]);

onMounted(async () => {
  accounts.value = await invoke<UserAccount[]>("get_user_accounts");
});
</script>

<template>
  <div class="ua-page">
    <NxSectionHeader title="Comptes utilisateurs" description="Comptes réels du système (lecture seule)." />
    <NxCard v-for="a in accounts" :key="a.username" class="ua-row">
      <strong>{{ a.username }}</strong> (UID {{ a.uid }}) — {{ a.home }} — {{ a.shell }}
    </NxCard>
  </div>
</template>

<style scoped>
.ua-page { padding: 24px; display: flex; flex-direction: column; gap: 10px; }
.ua-row { font-size: 13px; }
</style>
