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
  {
    id: "jeux",
    label: "Jeux",
    description: "Launchers et bibliothèques de jeux multi-plateformes.",
    appIds: ["steam", "lutris", "heroic", "retroarch"],
  },
  {
    id: "securite",
    label: "Sécurité & vie privée",
    description: "Mots de passe, chiffrement, VPN et protection contre les intrusions.",
    appIds: ["bitwarden", "gnupg", "wireguard", "fail2ban"],
  },
  {
    id: "serveur-web",
    label: "Serveur web",
    description: "Pile web classique et interface d'administration.",
    appIds: ["nginx", "mariadb-server", "php-fpm", "cockpit"],
  },
  {
    id: "science-education",
    label: "Science & Éducation",
    description: "Calcul numérique, notebooks interactifs, mathématiques et astronomie.",
    appIds: ["octave", "jupyter-notebook", "geogebra", "stellarium"],
  },
  {
    id: "accessibilite",
    label: "Accessibilité",
    description: "Lecteur d'écran, synthèse vocale, clavier virtuel et loupe.",
    appIds: ["orca", "espeak", "onboard", "kmag"],
  },
];
