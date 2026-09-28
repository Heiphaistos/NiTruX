//! Complete hardware inventory read from `/sys` and `/proc`, without root
//! and without external tools (except `lspci` for GPU names and
//! `nvidia-smi` for NVIDIA cards, both optional). What only root can read
//! (per-DIMM details, serial numbers) comes from `privileged_inventory`.

use crate::subprocess;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

fn kv(key: &str, value: impl Into<String>) -> KeyValue {
    KeyValue { key: key.to_string(), value: value.into() }
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Section {
    pub title: String,
    pub items: Vec<KeyValue>,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct Device {
    pub name: String,
    pub details: Vec<KeyValue>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct HardwareInventory {
    pub cpu: Section,
    pub cpu_cores: Vec<KeyValue>,
    pub firmware: Section,
    pub memory: Section,
    pub gpus: Vec<Device>,
    pub storage: Vec<Device>,
    pub power: Vec<Device>,
    pub network: Vec<Device>,
    pub usb: Vec<Device>,
    pub audio: Vec<Device>,
}

fn read(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn read_u64(path: impl AsRef<Path>) -> Option<u64> {
    read(path)?.parse().ok()
}

fn sorted_dir(path: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(path).map(|e| e.flatten().map(|e| e.path()).collect()).unwrap_or_default();
    v.sort();
    v
}

fn name_of(p: &Path) -> String {
    p.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default()
}

pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["o", "Ko", "Mo", "Go", "To", "Po"];
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 { format!("{bytes} o") } else { format!("{v:.1} {}", UNITS[i]) }
}

// ── CPU ────────────────────────────────────────────────────────────────────

/// Fields of the first processor block of /proc/cpuinfo, plus the flags
/// worth showing (virtualization, AES, AVX...).
pub fn parse_cpuinfo(content: &str) -> Vec<KeyValue> {
    let block = content.split("\n\n").next().unwrap_or("");
    let get = |k: &str| {
        block.lines().find_map(|l| {
            let (key, v) = l.split_once(':')?;
            (key.trim() == k).then(|| v.trim().to_string())
        })
    };
    let mut items = Vec::new();
    for (k, label) in [("model name", "Modèle"), ("Hardware", "Matériel"), ("vendor_id", "Fabricant"), ("cpu family", "Famille"), ("model", "Modèle (n°)"), ("stepping", "Stepping"), ("microcode", "Microcode"), ("cache size", "Cache")] {
        if let Some(v) = get(k) {
            items.push(kv(label, v));
        }
    }
    let flags = get("flags").or_else(|| get("Features")).unwrap_or_default();
    let has = |f: &str| flags.split_whitespace().any(|x| x == f);
    let mut notable = Vec::new();
    if has("vmx") { notable.push("VT-x"); }
    if has("svm") { notable.push("AMD-V"); }
    for (flag, name) in [("aes", "AES-NI"), ("sse4_2", "SSE4.2"), ("avx", "AVX"), ("avx2", "AVX2"), ("avx512f", "AVX-512"), ("sha_ni", "SHA"), ("rdrand", "RDRAND"), ("sme", "SME"), ("sev", "SEV"), ("neon", "NEON")] {
        if has(flag) {
            notable.push(name);
        }
    }
    if !notable.is_empty() {
        items.push(kv("Instructions notables", notable.join(", ")));
    }
    items.push(kv(
        "Virtualisation matérielle",
        if has("vmx") || has("svm") { "disponible" } else { "absente ou désactivée dans le BIOS" },
    ));
    if has("hypervisor") {
        items.push(kv("Environnement", "machine virtuelle (hyperviseur détecté)"));
    }
    items
}

fn read_cpu(sys: &Path, proc_: &Path) -> (Section, Vec<KeyValue>) {
    let mut items = read(proc_.join("cpuinfo")).map(|c| parse_cpuinfo(&c)).unwrap_or_default();
    let cpu_dir = sys.join("devices/system/cpu");
    let cores: Vec<PathBuf> = sorted_dir(&cpu_dir)
        .into_iter()
        .filter(|p| {
            let n = name_of(p);
            n.strip_prefix("cpu").is_some_and(|d| !d.is_empty() && d.chars().all(|c| c.is_ascii_digit()))
        })
        .collect();
    items.push(kv("Processeurs logiques", cores.len().to_string()));
    if let Some(online) = read(cpu_dir.join("online")) {
        items.push(kv("En ligne", online));
    }
    let freq0 = cpu_dir.join("cpu0/cpufreq");
    if let Some(g) = read(freq0.join("scaling_governor")) {
        items.push(kv("Gouverneur", g));
    }
    if let Some(d) = read(freq0.join("scaling_driver")) {
        items.push(kv("Pilote de fréquence", d));
    }
    if let (Some(min), Some(max)) = (read_u64(freq0.join("cpuinfo_min_freq")), read_u64(freq0.join("cpuinfo_max_freq"))) {
        items.push(kv("Fréquence min / max", format!("{} / {} MHz", min / 1000, max / 1000)));
    }
    if let Some(boost) = read(cpu_dir.join("cpufreq/boost")) {
        items.push(kv("Turbo / Boost", if boost == "1" { "activé" } else { "désactivé" }));
    }
    let mut mitig = Vec::new();
    for v in sorted_dir(&cpu_dir.join("vulnerabilities")) {
        if let Some(state) = read(&v) {
            if state.starts_with("Vulnerable") {
                mitig.push(name_of(&v));
            }
        }
    }
    items.push(kv(
        "Failles CPU non corrigées",
        if mitig.is_empty() { "aucune".to_string() } else { mitig.join(", ") },
    ));
    let per_core = cores
        .iter()
        .filter_map(|c| {
            let mhz = read_u64(c.join("cpufreq/scaling_cur_freq"))? / 1000;
            Some(kv(&name_of(c), format!("{mhz} MHz")))
        })
        .collect();
    (Section { title: "Processeur".into(), items }, per_core)
}

// ── Firmware / motherboard ─────────────────────────────────────────────────

pub fn chassis_name(code: &str) -> &'static str {
    match code.trim() {
        "3" => "Bureau",
        "4" => "Bureau bas profil",
        "6" => "Mini-tour",
        "7" => "Tour",
        "8" => "Portable",
        "9" => "Ordinateur portable",
        "10" => "Notebook",
        "13" => "Tout-en-un",
        "14" => "Sub-notebook",
        "17" => "Serveur (châssis principal)",
        "23" => "Serveur rack",
        "30" => "Tablette",
        "31" => "Convertible",
        "32" => "Détachable",
        "35" => "Mini PC",
        "36" => "Stick PC",
        "1" => "Autre",
        _ => "Inconnu",
    }
}

fn read_firmware(sys: &Path) -> Section {
    let dmi = sys.join("class/dmi/id");
    let mut items = Vec::new();
    for (file, label) in [
        ("sys_vendor", "Fabricant du système"),
        ("product_name", "Modèle"),
        ("product_version", "Version du modèle"),
        ("product_family", "Gamme"),
        ("product_sku", "SKU"),
        ("board_vendor", "Fabricant carte mère"),
        ("board_name", "Carte mère"),
        ("board_version", "Révision carte mère"),
        ("bios_vendor", "Fabricant BIOS/UEFI"),
        ("bios_version", "Version BIOS/UEFI"),
        ("bios_date", "Date BIOS/UEFI"),
        ("bios_release", "Release BIOS"),
        ("ec_firmware_release", "Firmware contrôleur embarqué"),
    ] {
        if let Some(v) = read(dmi.join(file)) {
            items.push(kv(label, v));
        }
    }
    if let Some(code) = read(dmi.join("chassis_type")) {
        items.push(kv("Type de châssis", chassis_name(&code)));
    }
    for (file, label) in [("product_serial", "N° de série"), ("board_serial", "N° de série carte mère"), ("product_uuid", "UUID système")] {
        items.push(kv(label, read(dmi.join(file)).unwrap_or_else(|| "lecture admin requise".into())));
    }
    let efi = sys.join("firmware/efi");
    items.push(kv("Mode de démarrage", if efi.exists() { "UEFI" } else { "BIOS (legacy)" }));
    if efi.exists() {
        if let Some(bits) = read(efi.join("fw_platform_size")) {
            items.push(kv("UEFI", format!("{bits} bits")));
        }
        items.push(kv("Secure Boot", secure_boot_state(&efi.join("efivars"))));
    }
    items.push(kv("TPM", tpm_state(&sys.join("class/tpm"))));
    Section { title: "Carte mère & firmware".into(), items }
}

/// The SecureBoot EFI variable is 4 attribute bytes then one data byte.
pub fn secure_boot_state(efivars: &Path) -> String {
    let var = efivars.join("SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c");
    match fs::read(&var) {
        Ok(bytes) if bytes.len() >= 5 => if bytes[4] == 1 { "activé".into() } else { "désactivé".into() },
        Ok(_) => "indéterminé".into(),
        Err(_) => "indéterminé (variable illisible)".into(),
    }
}

pub fn tpm_state(tpm_class: &Path) -> String {
    let tpm0 = tpm_class.join("tpm0");
    if !tpm0.exists() {
        return "absent ou désactivé dans le BIOS".into();
    }
    let version = read(tpm0.join("tpm_version_major")).map(|v| format!("TPM {v}.0"));
    let desc = read(tpm0.join("device/description")).or_else(|| read(tpm0.join("device/firmware_node/description")));
    match (version, desc) {
        (Some(v), Some(d)) => format!("présent — {v} ({d})"),
        (Some(v), None) => format!("présent — {v}"),
        (None, Some(d)) => format!("présent ({d})"),
        (None, None) => "présent".into(),
    }
}

// ── Memory ─────────────────────────────────────────────────────────────────

fn read_memory(proc_: &Path) -> Section {
    let mut items = Vec::new();
    if let Some(meminfo) = read(proc_.join("meminfo")) {
        let get = |k: &str| {
            meminfo.lines().find_map(|l| {
                let (key, v) = l.split_once(':')?;
                (key == k).then(|| v.trim().trim_end_matches(" kB").trim().parse::<u64>().ok()).flatten()
            })
        };
        for (k, label) in [("MemTotal", "Total utilisable"), ("MemAvailable", "Disponible"), ("Cached", "En cache"), ("SwapTotal", "Swap total"), ("SwapFree", "Swap libre"), ("HugePages_Total", "Huge pages")] {
            if let Some(v) = get(k) {
                let value = if k == "HugePages_Total" { v.to_string() } else { human_bytes(v * 1024) };
                items.push(kv(label, value));
            }
        }
    }
    items.push(kv("Barrettes (type, fréquence, fabricant)", "détail avec « Lire les détails (admin) »"));
    Section { title: "Mémoire".into(), items }
}

// ── GPUs ───────────────────────────────────────────────────────────────────

fn lspci_name(slot: &str) -> Option<String> {
    let out = subprocess::run_with_timeout("lspci", &["-s", slot], Duration::from_secs(5)).ok()?;
    let line = out.lines().next()?;
    Some(line.split_once(": ").map(|(_, d)| d.to_string()).unwrap_or_else(|| line.to_string()))
}

fn read_gpus(sys: &Path) -> Vec<Device> {
    let mut gpus = Vec::new();
    for card in sorted_dir(&sys.join("class/drm")) {
        let name = name_of(&card);
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let dev = card.join("device");
        let slot = fs::canonicalize(&dev).ok().map(|p| name_of(&p)).unwrap_or_default();
        let driver = fs::read_link(dev.join("driver")).ok().map(|p| name_of(&p)).unwrap_or_else(|| "aucun".into());
        let mut details = vec![kv("Pilote", driver.clone()), kv("Emplacement", slot.clone())];
        if let (Some(v), Some(d)) = (read(dev.join("vendor")), read(dev.join("device"))) {
            details.push(kv("ID PCI", format!("{}:{}", v.trim_start_matches("0x"), d.trim_start_matches("0x"))));
        }
        if let Some(total) = read_u64(dev.join("mem_info_vram_total")) {
            let used = read_u64(dev.join("mem_info_vram_used")).unwrap_or(0);
            details.push(kv("Mémoire vidéo (VRAM)", format!("{} utilisés / {}", human_bytes(used), human_bytes(total))));
        }
        if let Some(busy) = read(dev.join("gpu_busy_percent")) {
            details.push(kv("Charge GPU", format!("{busy} %")));
        }
        for (file, label) in [("pp_dpm_sclk", "Fréquence GPU"), ("pp_dpm_mclk", "Fréquence mémoire")] {
            if let Some(levels) = read(dev.join(file)) {
                if let Some(cur) = levels.lines().find(|l| l.ends_with('*')) {
                    details.push(kv(label, cur.split_whitespace().nth(1).unwrap_or(cur).to_string()));
                }
            }
        }
        if let (Some(speed), Some(width)) = (read(dev.join("current_link_speed")), read(dev.join("current_link_width"))) {
            let max = read(dev.join("max_link_speed")).map(|m| format!(" (max {m})")).unwrap_or_default();
            details.push(kv("Lien PCIe", format!("{speed} x{width}{max}")));
        }
        if let Some(vbios) = read(dev.join("vbios_version")) {
            details.push(kv("VBIOS", vbios));
        }
        let screens: Vec<String> = sorted_dir(&sys.join("class/drm"))
            .into_iter()
            .filter(|c| name_of(c).starts_with(&format!("{name}-")) && read(c.join("status")).as_deref() == Some("connected"))
            .map(|c| {
                let port = name_of(&c).trim_start_matches(&format!("{name}-")).to_string();
                match read(c.join("modes")).and_then(|m| m.lines().next().map(str::to_string)) {
                    Some(mode) => format!("{port} ({mode})"),
                    None => port,
                }
            })
            .collect();
        if !screens.is_empty() {
            details.push(kv("Écrans connectés", screens.join(", ")));
        }
        let title = lspci_name(&slot).unwrap_or_else(|| format!("Carte graphique {name}"));
        gpus.push(Device { name: title, details });
    }
    gpus.extend(nvidia_gpus());
    gpus
}

/// `nvidia-smi --query-gpu=... --format=csv,noheader,nounits`
pub fn parse_nvidia_query(output: &str) -> Vec<Device> {
    const FIELDS: [(&str, &str); 11] = [
        ("Pilote", ""), ("VRAM totale", " Mo"), ("VRAM utilisée", " Mo"), ("Charge GPU", " %"), ("Température", " °C"),
        ("Consommation", " W"), ("Limite de puissance", " W"), ("Ventilateur", " %"), ("Fréquence GPU", " MHz"),
        ("Fréquence mémoire", " MHz"), ("PCIe", ""),
    ];
    output
        .lines()
        .filter_map(|line| {
            let cols: Vec<&str> = line.split(',').map(str::trim).collect();
            if cols.len() < 1 + FIELDS.len() {
                return None;
            }
            let details = FIELDS
                .iter()
                .zip(&cols[1..])
                .filter(|(_, v)| !v.is_empty() && !v.contains("N/A") && !v.contains("Not Supported"))
                .map(|((k, unit), v)| kv(k, format!("{v}{unit}")))
                .collect();
            Some(Device { name: cols[0].to_string(), details })
        })
        .collect()
}

fn nvidia_gpus() -> Vec<Device> {
    if !subprocess::binary_in_path("nvidia-smi") {
        return Vec::new();
    }
    subprocess::run_with_timeout(
        "nvidia-smi",
        &[
            "--query-gpu=name,driver_version,memory.total,memory.used,utilization.gpu,temperature.gpu,power.draw,power.limit,fan.speed,clocks.gr,clocks.mem,pcie.link.gen.current",
            "--format=csv,noheader,nounits",
        ],
        Duration::from_secs(10),
    )
    .map(|o| {
        let mut devices = parse_nvidia_query(&o);
        for d in &mut devices {
            d.name = format!("{} (NVIDIA)", d.name);
        }
        devices
    })
    .unwrap_or_default()
}

// ── Storage ────────────────────────────────────────────────────────────────

fn read_storage(sys: &Path) -> Vec<Device> {
    let mut disks = Vec::new();
    for block in sorted_dir(&sys.join("block")) {
        let name = name_of(&block);
        if ["loop", "ram", "zram", "dm-", "md", "sr", "fd", "nbd"].iter().any(|p| name.starts_with(p)) {
            continue;
        }
        let size = read_u64(block.join("size")).unwrap_or(0) * 512;
        if size == 0 {
            continue;
        }
        let dev = block.join("device");
        let real = fs::canonicalize(&dev).map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
        let bus = if name.starts_with("nvme") {
            "NVMe"
        } else if real.contains("/usb") {
            "USB"
        } else if name.starts_with("mmcblk") {
            "carte SD / eMMC"
        } else if name.starts_with("vd") || real.contains("virtio") {
            "VirtIO (machine virtuelle)"
        } else {
            "SATA / SAS"
        };
        let rotational = read(block.join("queue/rotational")).as_deref() == Some("1");
        let kind = if name.starts_with("nvme") || (!rotational && bus != "VirtIO (machine virtuelle)") { "SSD" } else if rotational { "Disque dur (HDD)" } else { "Disque" };
        let model = read(dev.join("model")).or_else(|| read(dev.join("name"))).unwrap_or_else(|| name.clone());
        let mut details = vec![kv("Périphérique", format!("/dev/{name}")), kv("Capacité", human_bytes(size)), kv("Type", kind), kv("Interface", bus)];
        if let Some(v) = read(dev.join("vendor")) {
            details.push(kv("Fabricant", v));
        }
        for (file, label) in [("serial", "N° de série"), ("firmware_rev", "Firmware"), ("rev", "Révision")] {
            if let Some(v) = read(dev.join(file)) {
                details.push(kv(label, v));
            }
        }
        if read(block.join("removable")).as_deref() == Some("1") {
            details.push(kv("Amovible", "oui"));
        }
        if let Some(sched) = read(block.join("queue/scheduler")) {
            if let Some(active) = sched.split_whitespace().find(|s| s.starts_with('[')) {
                details.push(kv("Ordonnanceur E/S", active.trim_matches(['[', ']']).to_string()));
            }
        }
        let parts = sorted_dir(&block).into_iter().filter(|p| name_of(p).starts_with(&name)).count();
        details.push(kv("Partitions", parts.to_string()));
        disks.push(Device { name: model, details });
    }
    disks
}

// ── Power ──────────────────────────────────────────────────────────────────

fn read_power(sys: &Path) -> Vec<Device> {
    let mut out = Vec::new();
    for ps in sorted_dir(&sys.join("class/power_supply")) {
        let name = name_of(&ps);
        let kind = read(ps.join("type")).unwrap_or_default();
        let mut details = vec![kv("Type", match kind.as_str() {
            "Mains" => "Secteur (adaptateur)",
            "Battery" => "Batterie",
            "USB" => "USB (Power Delivery)",
            "UPS" => "Onduleur",
            other => other,
        })];
        if let Some(online) = read(ps.join("online")) {
            details.push(kv("Branché", if online == "1" { "oui" } else { "non" }));
        }
        for (file, label) in [("status", "État"), ("capacity", "Charge (%)"), ("capacity_level", "Niveau"), ("technology", "Technologie"), ("manufacturer", "Fabricant"), ("model_name", "Modèle"), ("serial_number", "N° de série"), ("cycle_count", "Cycles de charge")] {
            if let Some(v) = read(ps.join(file)) {
                details.push(kv(label, v));
            }
        }
        let full = read_u64(ps.join("energy_full")).or_else(|| read_u64(ps.join("charge_full")));
        let design = read_u64(ps.join("energy_full_design")).or_else(|| read_u64(ps.join("charge_full_design")));
        if let (Some(f), Some(d)) = (full, design) {
            if d > 0 {
                details.push(kv("Santé (capacité réelle / d'origine)", format!("{:.0} %", f as f64 / d as f64 * 100.0)));
            }
        }
        if let Some(p) = read_u64(ps.join("power_now")) {
            details.push(kv("Puissance instantanée", format!("{:.1} W", p as f64 / 1e6)));
        }
        if let Some(v) = read_u64(ps.join("voltage_now")) {
            details.push(kv("Tension", format!("{:.2} V", v as f64 / 1e6)));
        }
        out.push(Device { name, details });
    }
    out
}

// ── Network, USB, audio ────────────────────────────────────────────────────

fn read_network(sys: &Path) -> Vec<Device> {
    sorted_dir(&sys.join("class/net"))
        .into_iter()
        .filter(|n| n.join("device").exists())
        .map(|n| {
            let dev = n.join("device");
            let mut details = vec![kv("Interface", name_of(&n))];
            if let Some(d) = fs::read_link(dev.join("driver")).ok().map(|p| name_of(&p)) {
                details.push(kv("Pilote", d));
            }
            details.push(kv("Type", if n.join("wireless").exists() || n.join("phy80211").exists() { "Wi-Fi" } else { "Ethernet" }));
            for (file, label) in [("address", "Adresse MAC"), ("operstate", "État"), ("mtu", "MTU")] {
                if let Some(v) = read(n.join(file)) {
                    details.push(kv(label, v));
                }
            }
            if let Some(speed) = read(n.join("speed")).filter(|s| !s.starts_with('-')) {
                details.push(kv("Débit du lien", format!("{speed} Mb/s")));
            }
            let slot = fs::canonicalize(&dev).ok().map(|p| name_of(&p)).unwrap_or_default();
            let name = if slot.contains(':') { lspci_name(&slot) } else { None }
                .or_else(|| read(dev.join("../product")))
                .unwrap_or_else(|| name_of(&n));
            Device { name, details }
        })
        .collect()
}

fn read_usb(sys: &Path) -> Vec<Device> {
    sorted_dir(&sys.join("bus/usb/devices"))
        .into_iter()
        .filter(|d| d.join("idVendor").exists() && !name_of(d).starts_with("usb"))
        .map(|d| {
            let product = read(d.join("product")).unwrap_or_else(|| "Périphérique USB".into());
            let mut details = Vec::new();
            if let Some(m) = read(d.join("manufacturer")) {
                details.push(kv("Fabricant", m));
            }
            if let (Some(v), Some(p)) = (read(d.join("idVendor")), read(d.join("idProduct"))) {
                details.push(kv("ID", format!("{v}:{p}")));
            }
            if let Some(s) = read(d.join("speed")) {
                let label = match s.as_str() {
                    "1.5" => "USB 1.0 (1,5 Mb/s)".to_string(),
                    "12" => "USB 1.1 (12 Mb/s)".to_string(),
                    "480" => "USB 2.0 (480 Mb/s)".to_string(),
                    "5000" => "USB 3.0 (5 Gb/s)".to_string(),
                    "10000" => "USB 3.1 (10 Gb/s)".to_string(),
                    "20000" => "USB 3.2 (20 Gb/s)".to_string(),
                    other => format!("{other} Mb/s"),
                };
                details.push(kv("Vitesse", label));
            }
            if let Some(ma) = read(d.join("bMaxPower")) {
                details.push(kv("Consommation max", ma));
            }
            Device { name: product, details }
        })
        .collect()
}

pub fn parse_asound_cards(content: &str) -> Vec<Device> {
    content
        .lines()
        .filter_map(|l| {
            let l = l.trim_start();
            let (idx, rest) = l.split_once(' ')?;
            idx.parse::<u32>().ok()?;
            let (driver, name) = rest.split_once("]:").map(|(a, b)| (a, b.trim())).unwrap_or(("", rest));
            let driver = driver.trim().trim_start_matches('[').split_whitespace().next().unwrap_or("").to_string();
            Some(Device { name: name.split(" - ").last().unwrap_or(name).trim().to_string(), details: vec![kv("Pilote", driver)] })
        })
        .collect()
}

pub fn build_inventory(sys: &Path, proc_: &Path) -> HardwareInventory {
    let (cpu, cpu_cores) = read_cpu(sys, proc_);
    HardwareInventory {
        cpu,
        cpu_cores,
        firmware: read_firmware(sys),
        memory: read_memory(proc_),
        gpus: read_gpus(sys),
        storage: read_storage(sys),
        power: read_power(sys),
        network: read_network(sys),
        usb: read_usb(sys),
        audio: read(proc_.join("asound/cards")).map(|c| parse_asound_cards(&c)).unwrap_or_default(),
    }
}

#[tauri::command]
pub fn get_hardware_inventory() -> HardwareInventory {
    build_inventory(Path::new("/sys"), Path::new("/proc"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_bytes() {
        assert_eq!(human_bytes(512), "512 o");
        assert_eq!(human_bytes(16 * 1024 * 1024 * 1024), "16.0 Go");
    }

    #[test]
    fn parses_cpuinfo_flags_into_readable_capabilities() {
        let info = "processor\t: 0\nvendor_id\t: GenuineIntel\nmodel name\t: Intel(R) Core(TM) i7-8650U\nmicrocode\t: 0xf4\nflags\t\t: fpu vmx aes avx avx2 hypervisor\n\nprocessor\t: 1\n";
        let items = parse_cpuinfo(info);
        let get = |k: &str| items.iter().find(|i| i.key == k).map(|i| i.value.clone());
        assert_eq!(get("Modèle").as_deref(), Some("Intel(R) Core(TM) i7-8650U"));
        assert_eq!(get("Microcode").as_deref(), Some("0xf4"));
        assert_eq!(get("Instructions notables").as_deref(), Some("VT-x, AES-NI, AVX, AVX2"));
        assert_eq!(get("Virtualisation matérielle").as_deref(), Some("disponible"));
        assert!(get("Environnement").unwrap().contains("virtuelle"));
    }

    #[test]
    fn reads_secure_boot_and_tpm_from_fake_sysfs() {
        let root = std::env::temp_dir().join(format!("nitrux-fw-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("efivars")).unwrap();
        fs::write(root.join("efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c"), [6, 0, 0, 0, 1]).unwrap();
        assert_eq!(secure_boot_state(&root.join("efivars")), "activé");
        assert!(tpm_state(&root.join("tpm")).starts_with("absent"));
        fs::create_dir_all(root.join("tpm/tpm0")).unwrap();
        fs::write(root.join("tpm/tpm0/tpm_version_major"), "2\n").unwrap();
        assert_eq!(tpm_state(&root.join("tpm")), "présent — TPM 2.0");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn parses_nvidia_smi_query_output() {
        let out = "NVIDIA GeForce RTX 3070, 550.54.14, 8192, 1024, 12, 45, 35.20, 220.00, 30, 210, 405, 4\n";
        let d = parse_nvidia_query(out);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].name, "NVIDIA GeForce RTX 3070");
        assert!(d[0].details.iter().any(|k| k.key == "Température" && k.value == "45 °C"));
        assert!(d[0].details.iter().any(|k| k.key == "VRAM totale" && k.value == "8192 Mo"));
    }

    #[test]
    fn parses_asound_cards() {
        let c = " 0 [PCH            ]: HDA-Intel - HDA Intel PCH\n                      HDA Intel PCH at 0xf7f10000 irq 32\n 1 [NVidia         ]: HDA-Intel - HDA NVidia\n";
        let d = parse_asound_cards(c);
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].name, "HDA Intel PCH");
    }

    #[test]
    fn maps_chassis_codes() {
        assert_eq!(chassis_name("10"), "Notebook");
        assert_eq!(chassis_name("3"), "Bureau");
    }

    #[test]
    fn the_real_machine_inventory_has_a_cpu_and_a_disk() {
        let inv = get_hardware_inventory();
        assert!(inv.cpu.items.iter().any(|i| i.key == "Processeurs logiques"));
        assert!(!inv.storage.is_empty(), "this machine has at least one block device");
    }
}
