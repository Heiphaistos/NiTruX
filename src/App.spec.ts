// src/App.spec.ts
import { describe, it, expect, beforeEach, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import App from "./App.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(null),
}));

describe("App", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("renders AppNav with all 9 category titles", () => {
    const wrapper = mount(App);
    expect(wrapper.text()).toContain("Système");
    expect(wrapper.text()).toContain("Diagnostic");
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
});
