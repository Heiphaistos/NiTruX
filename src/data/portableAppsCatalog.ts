// Curated, deliberately small catalog: unlike appCatalog.ts (506 apt/
// flatpak/snap entries), a real AppImage download requires the
// publisher to actually ship one -- each entry here was individually
// verified live against the GitHub releases API (not guessed) to have
// exactly one unambiguous AppImage asset in its latest release before
// being added; see portable_apps.rs's own tests for the exact asset
// lists this was checked against.
export interface PortableAppEntry {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: string;
  githubOwner: string;
  githubRepo: string;
}

export const portableAppsCatalog: PortableAppEntry[] = [
  {
    id: "joplin",
    name: "Joplin",
    description: "Prise de notes et gestion de tâches avec chiffrement de bout en bout.",
    icon: "📝",
    category: "Productivité",
    githubOwner: "laurent22",
    githubRepo: "joplin",
  },
  {
    id: "obsidian",
    name: "Obsidian",
    description: "Prise de notes basée sur des fichiers Markdown liés entre eux.",
    icon: "🗒️",
    category: "Productivité",
    githubOwner: "obsidianmd",
    githubRepo: "obsidian-releases",
  },
  {
    id: "standard-notes",
    name: "Standard Notes",
    description: "Prise de notes chiffrée et simple, axée sur la confidentialité.",
    icon: "🔒",
    category: "Productivité",
    githubOwner: "standardnotes",
    githubRepo: "app",
  },
  {
    id: "keepassxc",
    name: "KeePassXC",
    description: "Gestionnaire de mots de passe hors ligne, chiffré.",
    icon: "🔐",
    category: "Sécurité",
    githubOwner: "keepassxreboot",
    githubRepo: "keepassxc",
  },
  {
    id: "copyq",
    name: "CopyQ",
    description: "Gestionnaire avancé de presse-papiers avec historique et éditeur.",
    icon: "📋",
    category: "Utilitaires",
    githubOwner: "hluk",
    githubRepo: "CopyQ",
  },
  {
    id: "cura",
    name: "UltiMaker Cura",
    description: "Logiciel de tranchage pour impression 3D.",
    icon: "🖨️",
    category: "Ingénierie",
    githubOwner: "Ultimaker",
    githubRepo: "Cura",
  },
  {
    id: "freecad",
    name: "FreeCAD",
    description: "Modeleur 3D paramétrique open source pour la conception mécanique.",
    icon: "📐",
    category: "Ingénierie",
    githubOwner: "FreeCAD",
    githubRepo: "FreeCAD",
  },
];
