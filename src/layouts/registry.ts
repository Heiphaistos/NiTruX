import type { LayoutDefinition } from "@/types/layout";

export const layoutRegistry: LayoutDefinition[] = [
  { id: "sidebar-classic", name: "Sidebar classique", description: "Navigation latérale fixe, zone de contenu principale." },
  { id: "widgets-grid", name: "Dashboard modulaire", description: "Grille de cartes réarrangeables sur l'accueil." },
  { id: "command-first", name: "Command palette-first", description: "Recherche/commande centrale, sidebar réduite." },
  { id: "compact-sidebar", name: "Sidebar rétractable", description: "Bande d'icônes, extension au survol." },
  { id: "top-nav", name: "Barre supérieure + onglets", description: "Navigation horizontale, contenu plein écran." },
  { id: "master-detail", name: "Master-detail", description: "Liste étroite à gauche, panneau détail à droite." },
  { id: "bento", name: "Bento grid", description: "Accueil en grille asymétrique." },
  { id: "floating-dock", name: "Dock flottant", description: "Contenu plein-bleed, navigation en dock flottant." },
];
