// src/data/systemToolsCatalog.ts
export interface SystemTool {
  id: string;
  name: string;
  description: string;
  category: "diagnostics" | "reseau" | "performance" | "nettoyage" | "stockage" | "privilegie";
  /** Exactly one of `command`/`privilegedAction` is set, never both. */
  command?: string;
  /** One of system_tools.rs's 7 fixed action names -- routed through
   *  run_system_tool (pkexec) instead of run_script. */
  privilegedAction?: string;
}

export const systemToolsCatalog: SystemTool[] = [
  // Diagnostics système
  { id: "uname", name: "Informations noyau", description: "Version du noyau et architecture.", category: "diagnostics", command: "uname -a" },
  { id: "uptime", name: "Disponibilité", description: "Depuis combien de temps le système tourne.", category: "diagnostics", command: "uptime -p" },
  { id: "free", name: "Mémoire", description: "Utilisation de la RAM et du swap.", category: "diagnostics", command: "LC_ALL=C free -h" },
  { id: "df", name: "Espace disque", description: "Espace utilisé/disponible par système de fichiers monté.", category: "diagnostics", command: "df -h" },
  { id: "lsblk", name: "Périphériques bloc", description: "Arborescence des disques et partitions.", category: "diagnostics", command: "lsblk" },
  { id: "lscpu", name: "Détails processeur", description: "Modèle, cœurs, threads, architecture.", category: "diagnostics", command: "LC_ALL=C lscpu" },
  { id: "whoami-id", name: "Identité courante", description: "Utilisateur et groupes actuels.", category: "diagnostics", command: "whoami && id" },
  { id: "hostnamectl", name: "Informations machine", description: "Nom d'hôte, type de machine, noyau, OS.", category: "diagnostics", command: "hostnamectl status" },
  { id: "timedatectl", name: "Date & heure", description: "Fuseau horaire et synchronisation NTP.", category: "diagnostics", command: "timedatectl status" },
  { id: "localectl", name: "Paramètres régionaux", description: "Langue système et disposition clavier.", category: "diagnostics", command: "localectl status" },
  { id: "w", name: "Sessions actives", description: "Utilisateurs connectés et leur activité.", category: "diagnostics", command: "w" },
  { id: "last", name: "Dernières connexions", description: "Historique des 10 dernières connexions.", category: "diagnostics", command: "last -n 10" },
  { id: "nproc", name: "Nombre de cœurs", description: "Nombre de cœurs/threads disponibles.", category: "diagnostics", command: "nproc" },
  { id: "failed-units", name: "Services en échec", description: "Services systemd actuellement en échec.", category: "diagnostics", command: "systemctl --failed --no-pager --plain" },
  { id: "journal-errors", name: "Erreurs récentes", description: "50 dernières erreurs du journal système (accès partiel possible selon vos groupes).", category: "diagnostics", command: "journalctl -p err -b --no-pager -n 50" },
  { id: "vmstat", name: "Statistiques mémoire/CPU", description: "Aperçu mémoire, swap et CPU sur 2 secondes.", category: "diagnostics", command: "LC_ALL=C vmstat 1 2" },
  { id: "sensors", name: "Capteurs matériels", description: "Températures et tensions (si lm-sensors installé).", category: "diagnostics", command: "sensors" },
  { id: "dmesg", name: "Journal noyau", description: "Messages du noyau (souvent restreint sans root).", category: "diagnostics", command: "dmesg" },
  { id: "os-release", name: "Version de la distribution", description: "Nom, version et code de la distribution installée.", category: "diagnostics", command: "cat /etc/os-release" },
  { id: "journal-disk-usage", name: "Taille des journaux", description: "Espace disque occupé par le journal système.", category: "diagnostics", command: "journalctl --disk-usage" },
  { id: "swap-status", name: "État du swap", description: "Partitions/fichiers de swap actifs et leur utilisation.", category: "diagnostics", command: "/usr/sbin/swapon --show" },
  { id: "findmnt", name: "Arborescence des montages", description: "Systèmes de fichiers montés, en arborescence.", category: "diagnostics", command: "findmnt" },

  // Réseau
  { id: "ip-a", name: "Interfaces réseau", description: "Liste des interfaces et adresses IP.", category: "reseau", command: "ip a" },
  { id: "ip-route", name: "Table de routage", description: "Routes réseau configurées.", category: "reseau", command: "ip route" },
  { id: "ip-link-stats", name: "Statistiques interfaces", description: "Compteurs de paquets/erreurs par interface.", category: "reseau", command: "ip -s link" },
  { id: "nmcli-device", name: "Périphériques réseau", description: "État des connexions via NetworkManager.", category: "reseau", command: "nmcli device status" },
  { id: "nmcli-connection", name: "Connexions configurées", description: "Liste des profils de connexion NetworkManager.", category: "reseau", command: "nmcli connection show" },
  { id: "ping", name: "Test de connectivité", description: "4 pings vers 8.8.8.8.", category: "reseau", command: "ping -c 4 8.8.8.8" },
  { id: "public-ip", name: "Adresse IP publique", description: "Votre IP publique actuelle via DNS OpenDNS.", category: "reseau", command: "dig +short myip.opendns.com @resolver1.opendns.com" },
  { id: "dig-google", name: "Résolution DNS (dig)", description: "Résout google.com via dig.", category: "reseau", command: "dig google.com" },
  { id: "host-google", name: "Résolution DNS (host)", description: "Résout google.com via host.", category: "reseau", command: "host google.com" },
  { id: "traceroute", name: "Traceroute", description: "Chemin réseau vers 8.8.8.8.", category: "reseau", command: "traceroute -m 15 8.8.8.8" },
  { id: "ss-ports", name: "Ports en écoute", description: "Sockets TCP/UDP en écoute.", category: "reseau", command: "ss -tulpn" },
  { id: "resolvectl", name: "État de la résolution DNS", description: "Résolveurs DNS actifs par interface (si systemd-resolved installé).", category: "reseau", command: "resolvectl status" },
  { id: "ip-neigh", name: "Table ARP/voisinage", description: "Adresses matérielles des hôtes voisins récemment contactés.", category: "reseau", command: "ip neigh" },
  { id: "resolv-conf", name: "Configuration DNS actuelle", description: "Contenu du fichier de résolution DNS.", category: "reseau", command: "cat /etc/resolv.conf" },

  // Performance
  { id: "ps-cpu", name: "Top processus CPU", description: "15 processus consommant le plus de CPU.", category: "performance", command: "ps aux --sort=-%cpu | head -15" },
  { id: "ps-mem", name: "Top processus mémoire", description: "15 processus consommant le plus de mémoire.", category: "performance", command: "ps aux --sort=-%mem | head -15" },
  { id: "top-snapshot", name: "Aperçu instantané", description: "Instantané top (sans rafraîchissement continu).", category: "performance", command: "top -bn1 | head -20" },
  { id: "loadavg", name: "Charge système", description: "Charge moyenne sur 1, 5 et 15 minutes.", category: "performance", command: "cat /proc/loadavg" },
  { id: "boot-time", name: "Temps de démarrage", description: "Durée du dernier démarrage (noyau + espace utilisateur).", category: "performance", command: "systemd-analyze" },
  { id: "boot-blame", name: "Services les plus lents au démarrage", description: "Classement des services par temps de démarrage.", category: "performance", command: "systemd-analyze blame | head -15" },

  // Nettoyage (utilisateur, non-privilégié)
  { id: "clean-thumbnails", name: "Vider le cache des miniatures", description: "Supprime les vignettes d'images mises en cache.", category: "nettoyage", command: "rm -rf ~/.cache/thumbnails/*" },
  { id: "clean-old-cache", name: "Nettoyer le cache ancien", description: "Supprime les fichiers de cache non touchés depuis 30 jours.", category: "nettoyage", command: "find ~/.cache -type f -atime +30 -delete" },
  { id: "cache-size", name: "Taille du cache", description: "Espace occupé par le dossier de cache utilisateur.", category: "nettoyage", command: "du -sh ~/.cache" },
  { id: "npm-cache-clean", name: "Vider le cache npm", description: "Nettoie le cache npm (si npm installé).", category: "nettoyage", command: "npm cache clean --force" },
  { id: "pip-cache-purge", name: "Vider le cache pip", description: "Nettoie le cache pip (si pip installé).", category: "nettoyage", command: "pip cache purge" },
  { id: "clean-fontcache", name: "Vider le cache des polices", description: "Supprime le cache de polices utilisateur (régénéré automatiquement).", category: "nettoyage", command: "rm -rf ~/.cache/fontconfig/*" },

  // Stockage
  { id: "lsblk-fs", name: "Systèmes de fichiers", description: "Disques avec type de système de fichiers et point de montage.", category: "stockage", command: "lsblk -f" },
  { id: "biggest-home-dirs", name: "Plus gros dossiers du home", description: "10 plus gros éléments de votre dossier personnel.", category: "stockage", command: "du -sh ~/* 2>/dev/null | sort -rh | head -10" },
  { id: "varlog-size", name: "Taille des journaux système", description: "Espace disque occupé par /var/log.", category: "stockage", command: "du -sh /var/log 2>/dev/null" },

  // Privilégié (root, via l'action polkit consolidée)
  { id: "apt-autoremove", name: "Retirer les paquets orphelins", description: "Supprime les dépendances devenues inutiles.", category: "privilegie", privilegedAction: "apt-autoremove" },
  { id: "journal-vacuum-size", name: "Réduire les journaux (taille)", description: "Limite le journal système à 200 Mo.", category: "privilegie", privilegedAction: "journal-vacuum-size" },
  { id: "rebuild-ld-cache", name: "Reconstruire le cache des bibliothèques", description: "Reconstruit le cache de l'éditeur de liens dynamique (ldconfig).", category: "privilegie", privilegedAction: "rebuild-ld-cache" },
  { id: "systemd-reload", name: "Recharger systemd", description: "Recharge les fichiers d'unités systemd modifiés.", category: "privilegie", privilegedAction: "systemd-reload" },
  { id: "fstrim-all", name: "TRIM des disques SSD", description: "Exécute TRIM sur tous les systèmes de fichiers compatibles.", category: "privilegie", privilegedAction: "fstrim-all" },
  { id: "rebuild-locate-db", name: "Reconstruire la base locate", description: "Met à jour la base de données utilisée par la commande locate.", category: "privilegie", privilegedAction: "rebuild-locate-db" },
  { id: "regenerate-grub", name: "Régénérer la configuration GRUB", description: "Régénère la configuration de démarrage après un changement de noyau.", category: "privilegie", privilegedAction: "regenerate-grub" },
];
