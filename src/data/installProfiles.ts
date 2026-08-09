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
    description: "Éditeur de code, contrôle de version, chaîne de compilation C/C++ et deux langages courants.",
    appIds: ["git", "vscode", "build-essential", "python3", "nodejs"],
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
  {
    id: "virtualisation",
    label: "Virtualisation & conteneurs",
    description: "Machines virtuelles et conteneurs pour isoler des environnements.",
    appIds: ["qemu-system", "virt-manager", "docker", "docker-compose"],
  },
  {
    id: "cloud-sync",
    label: "Cloud & Synchronisation",
    description: "Clients de synchronisation cloud et accès à des stockages distants.",
    appIds: ["nextcloud-desktop", "dropbox", "syncthing", "davfs2", "gigolo"],
  },
  {
    id: "productivite",
    label: "Productivité",
    description: "Prise de notes et gestion de tâches.",
    appIds: ["obsidian", "joplin", "taskwarrior", "planner"],
  },
  {
    id: "emulation",
    label: "Émulation & compatibilité",
    description: "Faire tourner d'anciens jeux et des logiciels Windows.",
    appIds: ["wine", "bottles", "dosbox", "scummvm", "dolphin-emu"],
  },
  {
    id: "partage-fichiers",
    label: "Partage de fichiers",
    description: "Clients BitTorrent, FTP et partage sur le réseau local.",
    appIds: ["transmission", "qbittorrent", "warpinator", "gftp"],
  },
];
