export interface NavPage {
  id: string;
  label: string;
  icon: string;
}

export interface NavCategory {
  id: string;
  title: string;
  pages: NavPage[];
}

export const navigationCategories: NavCategory[] = [
  {
    id: "systeme",
    title: "Système",
    pages: [
      { id: "dashboard", label: "Tableau de bord", icon: "layout-dashboard" },
    ],
  },
  {
    id: "diagnostic-avance",
    title: "Diagnostic",
    pages: [
      { id: "diagnostic", label: "Composants PCI", icon: "stethoscope" },
      { id: "hardware-details", label: "Matériel détaillé", icon: "cpu" },
      { id: "peripherals", label: "Périphériques", icon: "monitor" },
      { id: "processes", label: "Processus & services", icon: "activity" },
      { id: "installed-software", label: "Logiciels installés", icon: "list" },
      { id: "user-accounts", label: "Comptes utilisateurs", icon: "users" },
      { id: "update-history", label: "Historique des mises à jour", icon: "history" },
    ],
  },
  {
    id: "outils-systeme",
    title: "Outils système",
    pages: [
      { id: "system-tools", label: "Commandes rapides", icon: "terminal" },
    ],
  },
  {
    id: "performance",
    title: "Performance",
    pages: [
      { id: "optimizations", label: "Optimisations", icon: "zap" },
      { id: "temperatures", label: "Températures", icon: "thermometer" },
      { id: "benchmark", label: "Benchmark", icon: "gauge" },
      { id: "perf-history", label: "Historique perf.", icon: "bar-chart-3" },
    ],
  },
  {
    id: "applications",
    title: "Applications",
    pages: [
      { id: "quick-install", label: "Installation rapide", icon: "download" },
      { id: "package-manager", label: "Gestionnaire de paquets", icon: "package" },
      { id: "install-profiles", label: "Installation par profils", icon: "layers" },
    ],
  },
  {
    id: "stockage",
    title: "Stockage",
    pages: [
      { id: "disks", label: "Disques & partitions", icon: "hard-drive" },
      { id: "file-tools", label: "Doublons / Gros fichiers / Hash", icon: "files" },
      { id: "disk-visualizer", label: "Visualiseur de disque", icon: "pie-chart" },
      { id: "data-recovery", label: "Récupération de données", icon: "database" },
      { id: "boot-manager", label: "Boot Manager", icon: "server" },
      { id: "restore-points", label: "Restauration", icon: "shield-check" },
    ],
  },
  {
    id: "maintenance",
    title: "Maintenance",
    pages: [
      { id: "updates", label: "Mises à jour", icon: "refresh-cw" },
      { id: "drivers", label: "Pilotes", icon: "cpu" },
      { id: "troubleshoot", label: "Dépannage", icon: "wrench" },
      { id: "uninstaller", label: "Désinstalleur", icon: "trash-2" },
      { id: "cleaner", label: "Nettoyeur", icon: "sparkles" },
      { id: "backup", label: "Sauvegarde", icon: "save" },
      { id: "antivirus", label: "Antivirus", icon: "shield-check" },
      { id: "dependencies", label: "Dépendances", icon: "package-search" },
    ],
  },
  {
    id: "reseau",
    title: "Réseau",
    pages: [
      { id: "network-overview", label: "Vue d'ensemble", icon: "wifi" },
      { id: "dns-switcher", label: "DNS Switcher", icon: "globe" },
      { id: "wifi-analyzer", label: "WiFi Analyzer", icon: "radio" },
      { id: "bluetooth", label: "Bluetooth", icon: "bluetooth" },
      { id: "scripts", label: "Scripts & Snippets", icon: "file-code" },
      { id: "firewall", label: "Pare-feu", icon: "shield" },
    ],
  },
  {
    id: "rapports",
    title: "Rapports",
    pages: [
      { id: "report-generator", label: "Générateur de rapport", icon: "file-text" },
      { id: "logs", label: "Journaux", icon: "scroll-text" },
    ],
  },
  {
    id: "parametres",
    title: "Paramètres",
    pages: [
      { id: "settings-preferences", label: "Préférences", icon: "settings" },
      { id: "settings-appearance", label: "Thèmes & dispositions", icon: "palette" },
    ],
  },
];
