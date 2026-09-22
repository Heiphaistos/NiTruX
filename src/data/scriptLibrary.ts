// Ready-made shell snippets for the Scripts page.
//
// The page could save and run scripts, but started empty: someone who does
// not already know the commands had nothing to run. Every entry here is
// read-only and unprivileged -- it does exactly what the user could type in
// their own terminal, which is the documented boundary of `run_script`.
//
// Anything needing root belongs in the privileged, whitelisted
// `run_troubleshoot_action`/`run_system_tool` paths, not here.

export interface LibraryScript {
  name: string;
  description: string;
  category: string;
  content: string;
}

export const scriptLibrary: LibraryScript[] = [
  {
    name: "Top 10 des processus par mémoire",
    description: "Les dix processus qui consomment le plus de RAM.",
    category: "Système",
    content: "ps -eo pid,comm,%mem,%cpu --sort=-%mem | head -n 11",
  },
  {
    name: "Top 10 des processus par CPU",
    description: "Les dix processus qui consomment le plus de processeur.",
    category: "Système",
    content: "ps -eo pid,comm,%cpu,%mem --sort=-%cpu | head -n 11",
  },
  {
    name: "Services en échec",
    description: "Unités systemd dans l'état failed.",
    category: "Système",
    content: "systemctl --failed --no-pager",
  },
  {
    name: "Temps de démarrage",
    description: "Durée du dernier démarrage, puis les unités les plus lentes.",
    category: "Système",
    content: "systemd-analyze; echo; systemd-analyze blame | head -n 10",
  },
  {
    name: "Espace disque par point de montage",
    description: "Occupation des systèmes de fichiers réels, en unités lisibles.",
    category: "Stockage",
    content: "df -hT -x tmpfs -x devtmpfs -x squashfs",
  },
  {
    name: "Les 20 plus gros dossiers du dossier personnel",
    description: "Classement par taille, sans sortir du système de fichiers courant.",
    category: "Stockage",
    content: "du -xh -d 1 \"$HOME\" 2>/dev/null | sort -rh | head -n 20",
  },
  {
    name: "Inodes disponibles",
    description: "Un disque peut être plein d'inodes sans être plein d'octets.",
    category: "Stockage",
    content: "df -i -x tmpfs -x devtmpfs",
  },
  {
    name: "Ports en écoute",
    description: "Sockets TCP et UDP en écoute, avec le processus quand il est visible.",
    category: "Réseau",
    content: "ss -tulnp",
  },
  {
    name: "Connexions établies",
    description: "Connexions sortantes et entrantes actuellement ouvertes.",
    category: "Réseau",
    content: "ss -tunp state established",
  },
  {
    name: "Route par défaut et DNS",
    description: "Passerelle utilisée et serveurs de noms configurés.",
    category: "Réseau",
    content: "ip route show default; echo; cat /etc/resolv.conf",
  },
  {
    name: "Test de connectivité complet",
    description: "Passerelle, DNS public, puis résolution de nom : distingue une panne de réseau d'une panne de DNS.",
    category: "Réseau",
    content:
      "ping -c 2 -W 2 \"$(ip route show default | awk '{print $3; exit}')\" && ping -c 2 -W 2 1.1.1.1 && getent hosts example.com",
  },
  {
    name: "Dernières erreurs du journal",
    description: "50 dernières entrées de priorité erreur ou supérieure.",
    category: "Diagnostic",
    content: "journalctl -p err -n 50 --no-pager",
  },
  {
    name: "Erreurs du démarrage courant",
    description: "Uniquement les erreurs depuis le dernier démarrage.",
    category: "Diagnostic",
    content: "journalctl -p err -b --no-pager | tail -n 50",
  },
  {
    name: "Messages du noyau récents",
    description: "Tampon du noyau horodaté, utile après un branchement matériel.",
    category: "Diagnostic",
    content: "journalctl -k -n 50 --no-pager",
  },
  {
    name: "Processus tués faute de mémoire",
    description: "Traces de l'OOM killer dans le journal.",
    category: "Diagnostic",
    content: "journalctl -k --no-pager | grep -i 'out of memory' | tail -n 20 || echo 'aucun'",
  },
  {
    name: "Températures et capteurs",
    description: "Relevé brut de lm-sensors.",
    category: "Matériel",
    content: "sensors",
  },
  {
    name: "Matériel bloc et systèmes de fichiers",
    description: "Disques, partitions, types et points de montage.",
    category: "Matériel",
    content: "lsblk -o NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT",
  },
  {
    name: "Mémoire détaillée",
    description: "Utilisation RAM et swap, en unités lisibles.",
    category: "Matériel",
    content: "free -h; echo; swapon --show",
  },
  {
    name: "Paquets installés les plus lourds",
    description: "Classement par taille (apt/dpkg uniquement).",
    category: "Paquets",
    content:
      "dpkg-query -W -f='${Installed-Size}\\t${Package}\\n' 2>/dev/null | sort -rn | head -n 20 || echo 'dpkg absent sur ce système'",
  },
  {
    name: "Paquets installés manuellement",
    description: "Ce que vous avez demandé, par opposition aux dépendances tirées automatiquement.",
    category: "Paquets",
    content: "apt-mark showmanual 2>/dev/null | head -n 50 || echo 'apt absent sur ce système'",
  },
];

export const scriptLibraryCategories = [...new Set(scriptLibrary.map((s) => s.category))];
