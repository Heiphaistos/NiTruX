// src/data/appCatalog.ts
export type InstallMethod = "apt" | "flatpak" | "snap";

export interface AppCatalogEntry {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: string;
  installMethod: InstallMethod;
  /** Package name/id passed to the install command. For "apt" entries this
   *  is the package name used with whichever native manager is actually
   *  detected on the host (apt/dnf/pacman/zypper) — chosen from apps whose
   *  package name is consistent across those distros' default repos. If a
   *  given host happens to differ, `install_package` surfaces the real
   *  error from the package manager rather than failing silently. */
  packageId: string;
}

export const appCatalog: AppCatalogEntry[] = [
  { id: "firefox", name: "Firefox", description: "Navigateur web libre et rapide.", icon: "🦊", category: "Navigateurs", installMethod: "apt", packageId: "firefox" },
  { id: "chromium", name: "Chromium", description: "Navigateur web open source basé sur le projet Chromium.", icon: "🌐", category: "Navigateurs", installMethod: "apt", packageId: "chromium" },
  { id: "thunderbird", name: "Thunderbird", description: "Client de messagerie complet.", icon: "📧", category: "Communication", installMethod: "apt", packageId: "thunderbird" },
  { id: "discord", name: "Discord", description: "Messagerie vocale et textuelle pour communautés.", icon: "🎮", category: "Communication", installMethod: "flatpak", packageId: "com.discordapp.Discord" },
  { id: "libreoffice", name: "LibreOffice", description: "Suite bureautique complète (texte, tableur, présentation).", icon: "📄", category: "Bureautique", installMethod: "apt", packageId: "libreoffice" },
  { id: "gimp", name: "GIMP", description: "Éditeur d'images professionnel.", icon: "🎨", category: "Média", installMethod: "apt", packageId: "gimp" },
  { id: "inkscape", name: "Inkscape", description: "Éditeur de graphiques vectoriels.", icon: "✏️", category: "Média", installMethod: "apt", packageId: "inkscape" },
  { id: "blender", name: "Blender", description: "Suite de création 3D complète.", icon: "🧊", category: "Média", installMethod: "apt", packageId: "blender" },
  { id: "vlc", name: "VLC", description: "Lecteur multimédia universel.", icon: "🎬", category: "Média", installMethod: "apt", packageId: "vlc" },
  { id: "audacity", name: "Audacity", description: "Éditeur audio multipiste.", icon: "🎵", category: "Média", installMethod: "apt", packageId: "audacity" },
  { id: "obs-studio", name: "OBS Studio", description: "Capture et diffusion vidéo en direct.", icon: "📹", category: "Média", installMethod: "apt", packageId: "obs-studio" },
  { id: "spotify", name: "Spotify", description: "Streaming musical.", icon: "🎧", category: "Média", installMethod: "snap", packageId: "spotify" },
  { id: "steam", name: "Steam", description: "Plateforme de jeux vidéo.", icon: "🕹️", category: "Jeux", installMethod: "flatpak", packageId: "com.valvesoftware.Steam" },
  { id: "keepassxc", name: "KeePassXC", description: "Gestionnaire de mots de passe hors ligne.", icon: "🔐", category: "Utilitaires", installMethod: "apt", packageId: "keepassxc" },
  { id: "htop", name: "htop", description: "Moniteur de processus interactif en terminal.", icon: "📊", category: "Utilitaires", installMethod: "apt", packageId: "htop" },
  { id: "git", name: "Git", description: "Système de contrôle de version.", icon: "🔧", category: "Développement", installMethod: "apt", packageId: "git" },
];
