<!-- src/components/PkexecIntegrationBanner.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

// AppImage builds cannot run a post-install step (see
// pkexec_bootstrap.rs's module doc comment), so every privileged action
// (install/uninstall packages, troubleshoot, hosts/DNS, firewall, disk
// formatting, snapshots, quarantine) fails until this one-time,
// user-triggered setup runs. .deb/.rpm installs already have everything
// in place, so `needed` stays false and this banner never renders for
// them.
const needed = ref(false);
const dismissed = ref(false);
const installing = ref(false);
const installError = ref<string | null>(null);
const installSuccess = ref(false);

onMounted(async () => {
  // `is_pkexec_integration_installed` is infallible Rust (returns a bare
  // `bool`), but the IPC call itself can still reject at the Tauri layer --
  // the same rare-but-real category every other page in this app already
  // guards (see e.g. InstalledSoftwarePage.vue's identical note). Found live
  // via a browser-based check (never caught by a source read alone): with
  // no try/catch here, a rejection left `needed` stuck at its initial
  // `false` forever -- silently hiding the one banner that tells an
  // AppImage user their privileged actions are broken and how to fix it,
  // for the rest of the session. Defaulting to `true` on failure errs
  // toward showing the banner: a false positive costs one dismiss click (or
  // a harmless re-run of an idempotent install script) on a working .deb/
  // .rpm install, while a false negative leaves an AppImage user stuck with
  // no explanation at all.
  try {
    needed.value = !(await invoke<boolean>("is_pkexec_integration_installed"));
  } catch {
    needed.value = true;
  }
});

async function install() {
  installing.value = true;
  installError.value = null;
  try {
    await invoke<string>("install_pkexec_integration");
    installSuccess.value = true;
    needed.value = false;
  } catch (e) {
    installError.value = String(e);
  } finally {
    installing.value = false;
  }
}
</script>

<template>
  <div v-if="needed && !dismissed" class="pkb-banner">
    <div class="pkb-text">
      <strong>Fonctions privilégiées non activées.</strong>
      Cette version portable (AppImage) ne peut pas installer automatiquement les composants nécessaires aux actions
      nécessitant les droits administrateur (installation/désinstallation de paquets, dépannage, pare-feu, formatage
      de disque, etc.). Cliquez ci-dessous pour les activer en une fois — une seule demande de mot de passe.
    </div>
    <div class="pkb-actions">
      <button class="pkb-install" :disabled="installing" @click="install">
        {{ installing ? "Installation..." : "Activer les fonctions privilégiées" }}
      </button>
      <button class="pkb-dismiss" @click="dismissed = true">Plus tard</button>
    </div>
    <div v-if="installError" class="pkb-error">{{ installError }}</div>
  </div>
  <div v-else-if="installSuccess" class="pkb-success">Fonctions privilégiées activées avec succès.</div>
</template>

<style scoped>
.pkb-banner {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--nx-accent-warning);
  color: #1a1a1a;
  font-size: 13px;
}
.pkb-text { flex: 1; min-width: 260px; }
.pkb-actions { display: flex; gap: 8px; }
.pkb-install, .pkb-dismiss {
  padding: 8px 14px;
  border-radius: var(--nx-style-radius);
  border: none;
  cursor: pointer;
  font: inherit;
  font-weight: 600;
}
.pkb-install { background: #1a1a1a; color: white; }
.pkb-dismiss { background: transparent; color: #1a1a1a; text-decoration: underline; }
.pkb-error { width: 100%; font-size: 12px; color: #7f1d1d; }
/* Black, not white: computed WCAG contrast of white text against every
   built-in theme's accentSuccess fails all 13 (worst 1.37:1, best 3.16:1 --
   none clear the 4.5:1 AA minimum). Black clears all 13 cleanly (6.65:1 to
   15.30:1) -- same class of bug already fixed for NxButton--danger. */
.pkb-success {
  padding: 8px 16px;
  background: var(--nx-accent-success);
  color: black;
  font-size: 13px;
}
</style>
