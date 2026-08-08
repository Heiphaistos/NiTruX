<!-- src/pages/CertificatesPage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface CertEntry {
  filename: string;
  subject: string;
  issuer: string;
  not_after: string;
  is_expired: boolean;
  is_expiring_soon: boolean;
}

const certs = ref<CertEntry[] | null>(null);
const error = ref<string | null>(null);
const filter = ref("");

onMounted(async () => {
  try {
    certs.value = await invoke<CertEntry[]>("get_certificates");
  } catch (e) {
    error.value = String(e);
  }
});

const filtered = computed(() =>
  (certs.value ?? []).filter((c) => c.subject.toLowerCase().includes(filter.value.toLowerCase())),
);

const expiredCount = computed(() => (certs.value ?? []).filter((c) => c.is_expired).length);
const expiringSoonCount = computed(() => (certs.value ?? []).filter((c) => c.is_expiring_soon).length);

function status(cert: CertEntry): "danger" | "warning" | "success" {
  if (cert.is_expired) return "danger";
  if (cert.is_expiring_soon) return "warning";
  return "success";
}

function statusLabel(cert: CertEntry): string {
  if (cert.is_expired) return "Expiré";
  if (cert.is_expiring_soon) return "Expire bientôt";
  return "Valide";
}
</script>

<template>
  <div class="cert-page">
    <NxSectionHeader title="Certificats" description="Certificats d'autorité (CA) installés dans le magasin système." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <NxCard v-if="certs && (expiredCount > 0 || expiringSoonCount > 0)" danger>
      {{ expiredCount }} certificat(s) expiré(s), {{ expiringSoonCount }} expirant sous 30 jours.
    </NxCard>

    <NxCard>
      <NxSectionHeader :title="`Certificats (${certs?.length ?? 0})`" />
      <NxInput v-model="filter" placeholder="Filtrer par sujet..." aria-label="Filtrer les certificats par sujet" />
      <div v-for="c in filtered" :key="c.filename" class="cert-row">
        <div class="cert-info">
          <span class="cert-subject">{{ c.subject }}</span>
          <span class="cert-meta">{{ c.issuer }} · expire le {{ c.not_after }}</span>
        </div>
        <NxBadge :status="status(c)">{{ statusLabel(c) }}</NxBadge>
      </div>
      <div v-if="certs && filtered.length === 0" class="cert-empty">Aucun certificat ne correspond à cette recherche.</div>
    </NxCard>
  </div>
</template>

<style scoped>
.cert-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.cert-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 8px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
.cert-info { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.cert-subject { font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cert-meta { color: var(--nx-text-secondary); font-size: 11px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cert-empty { color: var(--nx-text-secondary); font-size: 13px; }
</style>
