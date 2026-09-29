<!-- src/pages/FirewallPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

// `rules_need_privilege`: `ufw status` is root-only, so for a normal user
// the backend reads the on/off state from ufw.conf and the rule list has
// to be fetched with authorization ("Afficher les règles").
interface FirewallStatus { active: boolean; rules: string[]; rules_need_privilege?: boolean }

const firewall = ref<FirewallStatus | null>(null);
const firewallError = ref<string | null>(null);
const loadingRules = ref(false);

async function loadFirewall() {
  firewallError.value = null;
  try {
    firewall.value = await invoke<FirewallStatus>("get_firewall_status");
  } catch (e) {
    firewallError.value = String(e);
  }
}

async function loadRulesPrivileged() {
  loadingRules.value = true;
  firewallError.value = null;
  try {
    firewall.value = await invoke<FirewallStatus>("get_firewall_rules_privileged");
  } catch (e) {
    firewallError.value = String(e);
  } finally {
    loadingRules.value = false;
  }
}

const toggling = ref(false);
const confirmingEnable = ref(false);

async function setEnabled(enabled: boolean) {
  toggling.value = true;
  confirmingEnable.value = false;
  firewallError.value = null;
  try {
    firewall.value = await invoke<FirewallStatus>("set_firewall_enabled", { enabled });
  } catch (e) {
    firewallError.value = String(e);
  } finally {
    toggling.value = false;
  }
}

// Rule editing used to exist only inside the Network overview page, so the
// page actually named "Pare-feu" could show rules but never change them.
const portProto = ref("");
const ruleBusy = ref(false);
const ruleError = ref<string | null>(null);
const ruleResult = ref<string | null>(null);

async function changeRule(command: "add_firewall_rule" | "remove_firewall_rule") {
  ruleBusy.value = true;
  ruleError.value = null;
  ruleResult.value = null;
  try {
    await invoke<string>(command, { portProto: portProto.value.trim() });
    ruleResult.value = command === "add_firewall_rule" ? "Règle ajoutée." : "Règle supprimée.";
    // The rule change was just authorized, so refreshing the list with the
    // privileged read normally reuses that authorization.
    await loadRulesPrivileged();
  } catch (e) {
    ruleError.value = String(e);
  } finally {
    ruleBusy.value = false;
  }
}

onMounted(loadFirewall);
</script>

<template>
  <div class="fw-page">
    <NxSectionHeader title="Pare-feu" description="État et règles UFW." />
    <NxCard v-if="firewallError" danger>{{ firewallError }}</NxCard>
    <template v-if="firewall">
      <div class="fw-status-row">
        <NxBadge :status="firewall.active ? 'success' : 'warning'">
          UFW {{ firewall.active ? "actif" : "inactif" }}
        </NxBadge>
        <NxButton :disabled="loadingRules" @click="loadRulesPrivileged">
          {{ loadingRules ? "Lecture..." : firewall.rules_need_privilege ? "Afficher les règles (admin)" : "Actualiser" }}
        </NxButton>
        <NxButton v-if="firewall.active" variant="danger" :disabled="toggling" @click="setEnabled(false)">
          {{ toggling ? "En cours..." : "Désactiver le pare-feu" }}
        </NxButton>
        <NxButton v-else-if="!confirmingEnable" :disabled="toggling" @click="confirmingEnable = true">Activer le pare-feu</NxButton>
      </div>
      <NxCard v-if="!firewall.active && confirmingEnable" class="fw-confirm">
        <p>
          Une fois activé, UFW bloque toute connexion entrante non autorisée. Si vous administrez cette machine à distance
          (SSH), autorisez d'abord le port <strong>22/tcp</strong> ci-dessous, sinon la connexion sera coupée.
        </p>
        <div class="fw-form-row">
          <NxButton :disabled="toggling" @click="setEnabled(true)">{{ toggling ? "Activation..." : "Confirmer l'activation" }}</NxButton>
          <NxButton variant="ghost" @click="confirmingEnable = false">Annuler</NxButton>
        </div>
      </NxCard>
      <p v-if="!firewall.active" class="fw-empty">
        Pare-feu inactif : les règles ajoutées sont enregistrées et s'appliqueront dès son activation.
      </p>
      <NxCard v-if="firewall.rules_need_privilege" class="fw-rules">
        <div class="fw-empty">La liste des règles n'est lisible qu'avec les droits administrateur.</div>
      </NxCard>
      <NxCard v-else-if="firewall.active" class="fw-rules">
        <div v-if="firewall.rules.length === 0" class="fw-empty">Aucune règle configurée.</div>
        <div v-for="(r, i) in firewall.rules" :key="i" class="fw-row">{{ r }}</div>
      </NxCard>
    </template>

    <NxCard class="fw-edit">
      <NxSectionHeader title="Autoriser ou retirer un port" />
      <div class="fw-form-row">
        <NxInput v-model="portProto" placeholder="ex: 8080/tcp" aria-label="Port et protocole" />
        <NxButton :disabled="ruleBusy || !portProto.trim()" @click="changeRule('add_firewall_rule')">Autoriser</NxButton>
        <NxButton variant="danger" :disabled="ruleBusy || !portProto.trim()" @click="changeRule('remove_firewall_rule')">Retirer</NxButton>
      </div>
      <NxCard v-if="ruleError" danger>{{ ruleError }}</NxCard>
      <NxBadge v-if="ruleResult" status="success" live>{{ ruleResult }}</NxBadge>
    </NxCard>
  </div>
</template>

<style scoped>
.fw-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.fw-status-row { display: flex; align-items: center; gap: 12px; }
.fw-rules { padding: 4px 16px; }
.fw-row { padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
.fw-row:last-child { border-bottom: none; }
.fw-empty { color: var(--nx-text-secondary); font-size: 13px; }
.fw-confirm p { margin: 0 0 10px; font-size: 13px; }
.fw-edit { display: flex; flex-direction: column; gap: 10px; }
.fw-form-row { display: flex; gap: 10px; align-items: center; }
</style>
