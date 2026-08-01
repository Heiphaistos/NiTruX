// src/data/installProfiles.ts
export interface InstallProfile {
  id: string;
  label: string;
  description: string;
  /** Ids referencing entries in appCatalog.ts -- never duplicated app data. */
  appIds: string[];
}

export const installProfiles: InstallProfile[] = [
  {
    id: "essentiels",
    label: "Essentiels",
    description: "Navigateur, bureautique, lecteur multimédia et mot de passe.",
    appIds: ["firefox", "libreoffice", "vlc", "keepassxc"],
  },
  {
    id: "developpement",
    label: "Développement",
    description: "Outils de base pour coder et gérer des versions.",
    appIds: ["git", "htop"],
  },
  {
    id: "creation",
    label: "Création & Média",
    description: "Édition d'image, audio, vidéo et création 3D.",
    appIds: ["gimp", "inkscape", "audacity", "obs-studio", "blender"],
  },
  {
    id: "communication",
    label: "Communication",
    description: "Messagerie et discussion.",
    appIds: ["thunderbird", "discord"],
  },
];
