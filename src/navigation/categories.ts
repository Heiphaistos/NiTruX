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
      { id: "diagnostic", label: "Diagnostic", icon: "stethoscope" },
    ],
  },
  {
    id: "applications",
    title: "Applications",
    pages: [
      { id: "quick-install", label: "Installation rapide", icon: "download" },
      { id: "package-manager", label: "Gestionnaire de paquets", icon: "package" },
    ],
  },
  {
    id: "stockage",
    title: "Stockage",
    pages: [
      { id: "disks", label: "Disques & partitions", icon: "hard-drive" },
      { id: "file-tools", label: "Doublons / Gros fichiers / Hash", icon: "files" },
    ],
  },
  {
    id: "maintenance",
    title: "Maintenance",
    pages: [
      { id: "updates", label: "Mises à jour", icon: "refresh-cw" },
      { id: "drivers", label: "Pilotes", icon: "cpu" },
      { id: "troubleshoot", label: "Dépannage", icon: "wrench" },
    ],
  },
  {
    id: "reseau",
    title: "Réseau",
    pages: [
      { id: "network-overview", label: "Vue d'ensemble", icon: "wifi" },
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
