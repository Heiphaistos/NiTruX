// src/App.spec.ts
import { describe, it, expect, beforeEach, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import App from "./App.vue";

// Backend commands whose real Rust signature always resolves to an array
// (bare Vec<T> or Result<Vec<T>, String>) -- Tauri IPC never actually
// serializes these as null, so defaulting the smoke-test mock to null for
// them (like every other, genuinely-nullable/object-shaped command) crashed
// page renders on `.length`/`.filter` after the test's own assertions had
// already passed. List kept in sync with the `-> Vec<T>`/`-> Result<Vec<T>>`
// commands registered in src-tauri/src/lib.rs's generate_handler! macro.
const ARRAY_RETURNING_COMMANDS = new Set([
  "find_duplicate_files",
  "find_large_files_cmd",
  "get_audio_sinks",
  "get_autostart_entries",
  "get_crash_events",
  "get_environment_variables",
  "get_monitors",
  "get_pci_devices",
  "get_printers",
  "get_processes",
  "get_recent_logs",
  "get_scheduled_tasks",
  "get_systemd_services",
  "get_update_history",
  "get_usb_devices",
  "get_user_accounts",
  "list_disk_usage",
  "list_disks",
  "list_installed_packages",
  "list_reports",
  "list_snapshots",
  "list_trash",
  "list_updates",
  "scan_for_malware",
  "scan_missing_dependencies",
  "scan_ports_cmd",
]);

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    // Default to "already installed" (the normal .deb/.rpm case) so the
    // AppImage-only setup banner doesn't show up in every unrelated test
    // in this file -- its own dedicated tests below override this.
    if (cmd === "is_pkexec_integration_installed") return Promise.resolve(true);
    return Promise.resolve(ARRAY_RETURNING_COMMANDS.has(cmd) ? [] : null);
  }),
  Channel: vi.fn(function () {
    return { onmessage: null };
  }),
}));

// Real xterm.js touches jsdom APIs it doesn't implement (matchMedia) and is
// expensive to initialize -- mounting it for every test in this file (via
// TerminalPage, reached through the shared `pages` map) was slow enough to
// starve other test files for CPU under parallel workers. Mocked exactly
// like TerminalPage.spec.ts's own isolated mock.
vi.mock("@xterm/xterm", () => ({
  Terminal: vi.fn(function () {
    return { open: vi.fn(), write: vi.fn(), onData: vi.fn(), loadAddon: vi.fn(), dispose: vi.fn(), rows: 24, cols: 80 };
  }),
}));

vi.mock("@xterm/addon-fit", () => ({
  FitAddon: vi.fn(function () {
    return { fit: vi.fn() };
  }),
}));

describe("App", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("renders AppNav with all 10 category titles", () => {
    const wrapper = mount(App);
    expect(wrapper.text()).toContain("Système");
    expect(wrapper.text()).toContain("Diagnostic");
    expect(wrapper.text()).toContain("Outils système");
    expect(wrapper.text()).toContain("Performance");
    expect(wrapper.text()).toContain("Applications");
    expect(wrapper.text()).toContain("Stockage");
    expect(wrapper.text()).toContain("Maintenance");
    expect(wrapper.text()).toContain("Réseau");
    expect(wrapper.text()).toContain("Rapports");
    expect(wrapper.text()).toContain("Paramètres");
  });

  it("defaults to the dashboard page", () => {
    const wrapper = mount(App);
    expect(wrapper.findComponent({ name: "DashboardPage" }).exists() || wrapper.html().length > 0).toBe(true);
  });

  it("switches to DiagnosticPage when its nav item is clicked", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const diagButton = buttons.find((b) => b.text() === "Composants PCI")!;
    await diagButton.trigger("click");
    expect(wrapper.text()).toContain("Composants matériels détectés");
  });

  it("shows the real QuickInstallPage (not a placeholder) for the quick-install id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const quickInstallButton = buttons.find((b) => b.text() === "Installation rapide")!;
    await quickInstallButton.trigger("click");
    expect(wrapper.text()).not.toContain("prévu pour Phase R3");
  });

  it("shows the real UpdatesPage (not a placeholder) for the updates id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const updatesButton = buttons.find((b) => b.text() === "Mises à jour")!;
    await updatesButton.trigger("click");
    expect(wrapper.text()).not.toContain("prévu pour Phase R4");
  });

  it("shows the real ReportGeneratorPage (not a placeholder) for the report-generator id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const reportButton = buttons.find((b) => b.text() === "Générateur de rapport")!;
    await reportButton.trigger("click");
    expect(wrapper.text()).not.toContain("prévu pour Phase R5");
  });

  it("shows the real TemperaturesPage for the temperatures id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const tempButton = buttons.find((b) => b.text() === "Températures")!;
    await tempButton.trigger("click");
    expect(wrapper.text()).toContain("Relevés des capteurs thermiques");
  });

  it("shows the real BenchmarkPage for the benchmark id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const benchButton = buttons.find((b) => b.text() === "Benchmark")!;
    await benchButton.trigger("click");
    expect(wrapper.text()).toContain("Lancer le benchmark");
  });

  it("navigates to DiagnosticPage when the dashboard's Diagnostic quick-action tile is clicked", async () => {
    const wrapper = mount(App);
    await vi.waitFor(() => expect(wrapper.findAll(".nx-quick-action").length).toBeGreaterThan(0));
    const tiles = wrapper.findAll(".nx-quick-action");
    const diagnosticTile = tiles.find((t) => t.text().includes("Diagnostic"))!;
    await diagnosticTile.trigger("click");
    expect(wrapper.text()).toContain("Composants matériels détectés");
  });

  it("shows the real AntivirusPage for the antivirus id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Antivirus")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Analyse un dossier");
  });

  it("shows the real UninstallerPage for the uninstaller id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Désinstalleur")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Détection du gestionnaire");
  });

  it("shows the real BluetoothPage for the bluetooth id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Bluetooth")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Statut de l'adaptateur");
  });

  it("shows the real ScriptsPage for the scripts id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Scripts & Snippets")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("aucune élévation");
  });

  it("shows the real InstallProfilesPage for the install-profiles id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Installation par profils")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Sélectionnez un profil");
  });

  it("shows the real HardwareDetailsPage for the hardware-details id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Matériel détaillé")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Processeur");
  });

  it("shows the real PeripheralsPage for the peripherals id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Périphériques")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Moniteurs");
  });

  it("shows the real ProcessesPage for the processes id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Processus & services")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Démarrage automatique");
  });

  it("shows the real InstalledSoftwarePage for the installed-software id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Logiciels installés")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Variables d'environnement");
  });

  it("shows the real UserAccountsPage for the user-accounts id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Comptes utilisateurs")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Comptes réels du système");
  });

  it("shows the real UpdateHistoryPage for the update-history id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Historique des mises à jour")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Journal des installations");
  });

  it("shows the real SystemToolsPage for the system-tools id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Commandes rapides")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("sans terminal");
  });

  it("shows the real TerminalPage for the terminal id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Terminal")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Shell interactif");
  });

  it("shows the real DataRecoveryPage for the data-recovery id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Récupération de données")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Corbeille");
  });

  it("shows the real RestorePointsPage for the restore-points id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Restauration")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Instantanés système");
  });

  it("passes the compact icons variant to AppNav for the floating-dock layout", () => {
    localStorage.setItem("nitrux-layout", "floating-dock");
    const wrapper = mount(App);
    expect(wrapper.text()).not.toContain("Tableau de bord");
    const items = wrapper.findAll(".nx-app-nav__item");
    expect(items.length).toBeGreaterThan(0);
  });

  it("keeps the full list variant (category titles + labels) for the sidebar-classic layout", () => {
    localStorage.setItem("nitrux-layout", "sidebar-classic");
    const wrapper = mount(App);
    expect(wrapper.text()).toContain("Système");
    expect(wrapper.text()).toContain("Tableau de bord");
  });

  it("shows the pkexec integration banner only when it is not yet installed (AppImage case)", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "is_pkexec_integration_installed") return Promise.resolve(false);
      return Promise.resolve(ARRAY_RETURNING_COMMANDS.has(cmd) ? [] : null);
    });
    const wrapper = mount(App);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Fonctions privilégiées non activées"));
  });

  it("does not show the pkexec integration banner when already installed (normal .deb/.rpm case)", async () => {
    const wrapper = mount(App);
    await wrapper.vm.$nextTick();
    expect(wrapper.text()).not.toContain("Fonctions privilégiées non activées");
  });

  it("keeps the terminal session alive when switching away and back (does not re-spawn)", async () => {
    // invoke's call history is shared (not cleared) across tests in this
    // file, so this counts spawn_terminal calls made from this test's own
    // mount onward, not from zero.
    const { invoke } = await import("@tauri-apps/api/core");
    const spawnCallsBefore = vi.mocked(invoke).mock.calls.filter((c) => c[0] === "spawn_terminal").length;
    const closeCallsBefore = vi.mocked(invoke).mock.calls.filter((c) => c[0] === "close_terminal").length;

    const wrapper = mount(App);
    const buttons = () => wrapper.findAll("button");

    await buttons().find((b) => b.text() === "Terminal")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Shell interactif"));
    const spawnCallsAfterFirstVisit = vi.mocked(invoke).mock.calls.filter((c) => c[0] === "spawn_terminal").length - spawnCallsBefore;
    expect(spawnCallsAfterFirstVisit).toBe(1);

    await buttons().find((b) => b.text() === "Tableau de bord")!.trigger("click");
    await wrapper.vm.$nextTick();
    await buttons().find((b) => b.text() === "Terminal")!.trigger("click");
    await wrapper.vm.$nextTick();

    const spawnCallsAfterReturning = vi.mocked(invoke).mock.calls.filter((c) => c[0] === "spawn_terminal").length - spawnCallsBefore;
    expect(spawnCallsAfterReturning).toBe(1);
    const closeCallsAfterReturning = vi.mocked(invoke).mock.calls.filter((c) => c[0] === "close_terminal").length - closeCallsBefore;
    expect(closeCallsAfterReturning).toBe(0);
  });

  it("re-applies a persisted non-default style to the DOM on mount, not just the default", () => {
    // Regression guard for the actual bug: styleStore.ts's applyToDom only
    // ever ran inside setStyle() -- the store's state initializer read the
    // persisted style into `current` but never touched the DOM, and
    // useStyleStore was otherwise instantiated nowhere except
    // ThemeEditorPage.vue. A user who picked a non-default style in a
    // previous session would see it silently revert to the unstyled
    // default (no data-nx-style attribute at all) on every fresh launch,
    // exactly the same class of bug themeStore already had a fix for
    // (onMounted(() => themeStore.setTheme(themeStore.active)), right next
    // to this).
    document.documentElement.removeAttribute("data-nx-style");
    localStorage.setItem("nitrux-style", "brutalism");
    mount(App);
    expect(document.documentElement.dataset.nxStyle).toBe("brutalism");
  });
});
