//! Universal app store: which installation sources exist on this machine,
//! a live search across all of them, and one-click activation of Flatpak
//! and Snap. Searching live on the user's own machine (instead of a fixed
//! name table) is what makes one catalog work on every distribution: the
//! same app is `firefox-esr` on Debian, `firefox` elsewhere, and only on
//! Flathub on others.

use super::{binary_exists, detect_package_managers};
use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

const FLATHUB_REPO_URL: &str = "https://flathub.org/repo/flathub.flatpakrepo";
const SEARCH_LIMIT_PER_SOURCE: usize = 40;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct InstallSources {
    /// First detected native manager (apt, dnf, pacman, zypper, apk, xbps).
    pub native: Option<String>,
    pub flatpak: bool,
    /// Flathub configured (system-wide or for this user).
    pub flathub: bool,
    pub snap: bool,
    /// Only meaningful when `snap` is true: the daemon answers.
    pub snapd_running: bool,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct SearchResult {
    /// "native" (see `manager`), "flatpak" or "snap".
    pub source: String,
    /// Native manager id when `source` is "native".
    pub manager: Option<String>,
    /// What to pass to the install command.
    pub id: String,
    pub name: String,
    pub summary: String,
    pub version: String,
}

fn result(source: &str, manager: Option<&str>, id: &str, name: &str, summary: &str, version: &str) -> SearchResult {
    SearchResult {
        source: source.to_string(),
        manager: manager.map(str::to_string),
        id: id.trim().to_string(),
        name: name.trim().to_string(),
        summary: summary.trim().to_string(),
        version: version.trim().to_string(),
    }
}

/// Search terms reach external tools as one argv entry; still, only
/// letters, digits and a few separators are accepted, so a term can never
/// be read as an option or a pattern with surprising cost.
pub fn validate_query(query: &str) -> Result<String, String> {
    let q = query.trim();
    if q.is_empty() || q.len() > 64 {
        return Err("recherche vide ou trop longue (64 caractères max)".to_string());
    }
    if q.starts_with('-') || !q.chars().all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '.' | '+' | '_')) {
        return Err(format!("recherche invalide : {q}"));
    }
    Ok(q.to_string())
}

// ── Parsers, one per tool output ───────────────────────────────────────────

/// `apt-cache search --names-only`: `name - summary`.
pub fn parse_apt_search(output: &str) -> Vec<SearchResult> {
    output
        .lines()
        .filter_map(|l| l.split_once(" - "))
        .map(|(n, s)| result("native", Some("apt"), n, n, s, ""))
        .collect()
}

/// `dnf search`: dnf4 prints `name.arch : summary`, dnf5 ` name.arch\tsummary`,
/// both with section header lines that do not match either shape.
pub fn parse_dnf_search(output: &str) -> Vec<SearchResult> {
    output
        .lines()
        .filter_map(|l| l.split_once(" : ").or_else(|| l.trim_start().split_once('\t')))
        .filter_map(|(n, s)| {
            let n = n.trim();
            let name = n.rsplit_once('.').map(|(name, _arch)| name).unwrap_or(n);
            (!name.is_empty() && !name.contains(' ')).then(|| result("native", Some("dnf"), name, name, s, ""))
        })
        .collect()
}

/// `pacman -Ss`: `repo/name version [flags]` followed by an indented summary.
pub fn parse_pacman_search(output: &str) -> Vec<SearchResult> {
    let mut out = Vec::new();
    let mut lines = output.lines().peekable();
    while let Some(line) = lines.next() {
        if line.starts_with(' ') {
            continue;
        }
        let mut fields = line.split_whitespace();
        let (Some(full), Some(version)) = (fields.next(), fields.next()) else { continue };
        let name = full.rsplit('/').next().unwrap_or(full);
        let summary = match lines.peek() {
            Some(next) if next.starts_with(' ') => lines.next().unwrap_or_default(),
            _ => "",
        };
        out.push(result("native", Some("pacman"), name, name, summary, version));
    }
    out
}

/// `zypper -q search`: a `|`-separated table `S | Name | Summary | Type`.
pub fn parse_zypper_search(output: &str) -> Vec<SearchResult> {
    output
        .lines()
        .filter_map(|l| {
            let cols: Vec<&str> = l.split('|').map(str::trim).collect();
            (cols.len() >= 4 && cols[1] != "Name" && cols[3] == "package").then(|| result("native", Some("zypper"), cols[1], cols[1], cols[2], ""))
        })
        .collect()
}

/// `apk search -v -d`: `name-ver-rN - description`.
pub fn parse_apk_search(output: &str) -> Vec<SearchResult> {
    output
        .lines()
        .filter_map(|l| {
            let (pkgver, summary) = l.split_once(" - ")?;
            let (name, version) = super::apk::split_apk_pkgver(pkgver.trim())?;
            Some(result("native", Some("apk"), &name, &name, summary, &version))
        })
        .collect()
}

/// `xbps-query -Rs`: `[-] name-ver_rev   summary`.
pub fn parse_xbps_search(output: &str) -> Vec<SearchResult> {
    output
        .lines()
        .filter_map(|l| {
            let rest = l.trim_start().strip_prefix('[')?.split_once(']')?.1.trim_start();
            let (pkgver, summary) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
            let (name, version) = super::xbps::split_xbps_pkgver(pkgver)?;
            Some(result("native", Some("xbps"), &name, &name, summary, &version))
        })
        .collect()
}

/// `flatpak search --columns=application,name,description,version`: tab-separated.
pub fn parse_flatpak_search(output: &str) -> Vec<SearchResult> {
    output
        .lines()
        .filter_map(|l| {
            let cols: Vec<&str> = l.split('\t').collect();
            (cols.len() >= 3 && cols[0].contains('.')).then(|| {
                result("flatpak", None, cols[0], cols[1], cols[2], cols.get(3).copied().unwrap_or(""))
            })
        })
        .collect()
}

/// `snap find`: columns separated by runs of spaces:
/// `Name  Version  Publisher  Notes  Summary`.
pub fn parse_snap_find(output: &str) -> Vec<SearchResult> {
    output
        .lines()
        .skip_while(|l| !l.starts_with("Name"))
        .skip(1)
        .filter_map(|l| {
            let cols: Vec<&str> = l.split("  ").map(str::trim).filter(|c| !c.is_empty()).collect();
            (cols.len() >= 4).then(|| result("snap", None, cols[0], cols[0], cols[cols.len() - 1], cols[1]))
        })
        .collect()
}

// ── Commands ───────────────────────────────────────────────────────────────

fn flathub_configured() -> bool {
    subprocess::run_with_timeout("flatpak", &["remotes", "--columns=name"], Duration::from_secs(10))
        .map(|out| out.lines().any(|l| l.trim() == "flathub"))
        .unwrap_or(false)
}

#[tauri::command]
pub fn get_install_sources() -> InstallSources {
    let native = detect_package_managers().first().map(|m| m.id().to_string());
    let flatpak = binary_exists("flatpak");
    let snap = binary_exists("snap");
    InstallSources {
        native,
        flatpak,
        flathub: flatpak && flathub_configured(),
        snap,
        snapd_running: snap && subprocess::run_with_timeout("snap", &["version"], Duration::from_secs(10))
            .map(|out| out.lines().any(|l| l.starts_with("snapd") && !l.contains("unavailable")))
            .unwrap_or(false),
    }
}

fn native_search(manager: &str, q: &str) -> Result<Vec<SearchResult>, String> {
    let t = Duration::from_secs(30);
    Ok(match manager {
        "apt" => parse_apt_search(&subprocess::run_with_timeout("apt-cache", &["search", "--names-only", q], t)?),
        "dnf" => parse_dnf_search(&subprocess::run_with_timeout("dnf", &["-q", "search", q], t)?),
        "pacman" => parse_pacman_search(&subprocess::run_with_timeout("pacman", &["-Ss", q], t).or_else(|e| {
            // pacman -Ss exits 1 when nothing matches.
            if e.contains("code 1)") { Ok(String::new()) } else { Err(e) }
        })?),
        "zypper" => parse_zypper_search(&subprocess::run_with_timeout("zypper", &["-q", "search", q], t).or_else(|e| {
            // 104 = ZYPPER_EXIT_INF_CAP_NOT_FOUND: no match.
            if e.contains("code 104)") { Ok(String::new()) } else { Err(e) }
        })?),
        "apk" => parse_apk_search(&subprocess::run_with_timeout("apk", &["search", "-v", "-d", q], t)?),
        "xbps" => parse_xbps_search(&subprocess::run_with_timeout("xbps-query", &["-Rs", q], t)?),
        other => return Err(format!("recherche non prise en charge pour {other}")),
    })
}

/// Ranks exact and prefix name matches first: a search for "firefox" must
/// show `firefox` before `firefox-l10n-fr` and `webext-...-firefox`.
pub fn rank(results: &mut [SearchResult], q: &str) {
    let q = q.to_lowercase();
    results.sort_by_key(|r| {
        let (id, name) = (r.id.to_lowercase(), r.name.to_lowercase());
        let last = id.rsplit('.').next().unwrap_or(&id).to_string();
        if name == q || id == q || last == q {
            0
        } else if name.starts_with(&q) || id.starts_with(&q) || last.starts_with(&q) {
            1
        } else if name.contains(&q) {
            2
        } else {
            3
        }
    });
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct SearchOutcome {
    pub results: Vec<SearchResult>,
    /// Per-source failures ("flatpak : ..."): one source failing never
    /// hides the others' results.
    pub errors: Vec<String>,
}

/// Searches every available source in parallel.
#[tauri::command]
pub fn search_packages(query: String) -> Result<SearchOutcome, String> {
    let q = validate_query(&query)?;
    let sources = get_install_sources();
    let mut handles = Vec::new();
    if let Some(manager) = sources.native.clone() {
        let q = q.clone();
        handles.push(std::thread::spawn(move || (manager.clone(), native_search(&manager, &q))));
    }
    if sources.flatpak {
        let q = q.clone();
        handles.push(std::thread::spawn(move || {
            let out = subprocess::run_with_timeout(
                "flatpak",
                &["search", "--columns=application,name,description,version", &q],
                Duration::from_secs(30),
            );
            ("flatpak".to_string(), out.map(|o| parse_flatpak_search(&o)))
        }));
    }
    if sources.snap && sources.snapd_running {
        let q = q.clone();
        handles.push(std::thread::spawn(move || {
            let out = subprocess::run_with_timeout("snap", &["find", &q], Duration::from_secs(30)).or_else(|e| {
                // "No matching snaps" is reported with a non-zero exit.
                if e.contains("No matching snaps") { Ok(String::new()) } else { Err(e) }
            });
            ("snap".to_string(), out.map(|o| parse_snap_find(&o)))
        }));
    }
    let mut outcome = SearchOutcome { results: Vec::new(), errors: Vec::new() };
    for handle in handles {
        match handle.join() {
            Ok((_, Ok(mut found))) => {
                rank(&mut found, &q);
                found.truncate(SEARCH_LIMIT_PER_SOURCE);
                outcome.results.extend(found);
            }
            Ok((source, Err(e))) => outcome.errors.push(format!("{source} : {e}")),
            Err(_) => outcome.errors.push("une recherche a planté".to_string()),
        }
    }
    rank(&mut outcome.results, &q);
    Ok(outcome)
}

/// Installs Flatpak itself when missing (through the native manager and the
/// existing install-package pkexec action), then adds Flathub for the user.
#[tauri::command]
pub fn setup_flatpak() -> Result<String, String> {
    let mut log = String::new();
    if !binary_exists("flatpak") {
        let manager = detect_package_managers()
            .first()
            .map(|m| m.id().to_string())
            .ok_or("aucun gestionnaire de paquets natif pour installer flatpak")?;
        log.push_str(&super::install::install_package(manager, "flatpak".to_string())?);
    }
    log.push_str(&subprocess::run_with_timeout(
        "flatpak",
        &["remote-add", "--if-not-exists", "--user", "flathub", FLATHUB_REPO_URL],
        Duration::from_secs(60),
    )?);
    Ok(if log.trim().is_empty() { "Flatpak et Flathub sont prêts.".to_string() } else { log })
}

/// Installs snapd when missing, then enables its socket (pkexec).
#[tauri::command]
pub fn setup_snap() -> Result<String, String> {
    let mut log = String::new();
    if !binary_exists("snap") {
        let manager = detect_package_managers()
            .first()
            .map(|m| m.id().to_string())
            .ok_or("aucun gestionnaire de paquets natif pour installer snapd")?;
        log.push_str(&super::install::install_package(manager, "snapd".to_string())?);
    }
    log.push_str(&subprocess::run_with_timeout(
        "pkexec",
        &["/usr/bin/nitrux-pkexec-install-snap", "enable-snapd"],
        Duration::from_secs(120),
    )?);
    Ok(if log.trim().is_empty() { "Snap est prêt.".to_string() } else { log })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_queries() {
        assert_eq!(validate_query("  vlc ").unwrap(), "vlc");
        assert!(validate_query("").is_err());
        assert!(validate_query("--help").is_err());
        assert!(validate_query("a;rm -rf /").is_err());
        assert!(validate_query(&"a".repeat(65)).is_err());
    }

    #[test]
    fn parses_apt_and_dnf_outputs() {
        let apt = parse_apt_search("vlc - multimedia player and streamer\nvlc-bin - binaries from VLC\n");
        assert_eq!(apt.len(), 2);
        assert_eq!(apt[0].id, "vlc");
        let dnf4 = parse_dnf_search("========= Name Exactly Matched: vlc =========\nvlc.x86_64 : The cross-platform open-source multimedia framework\n");
        assert_eq!(dnf4.len(), 1);
        assert_eq!(dnf4[0].id, "vlc");
        let dnf5 = parse_dnf_search("Matched fields: name (exact)\n vlc.x86_64\tThe cross-platform multimedia framework\n");
        assert_eq!(dnf5[0].id, "vlc");
    }

    #[test]
    fn parses_pacman_output_with_summaries() {
        let out = "extra/vlc 3.0.20-8\n    Multi-platform MPEG, VCD/DVD, and DivX player\nextra/vlc-plugins-all 3.0.20-8 [installed]\n    all plugins\n";
        let r = parse_pacman_search(out);
        assert_eq!(r.len(), 2);
        assert_eq!((r[0].id.as_str(), r[0].version.as_str()), ("vlc", "3.0.20-8"));
        assert!(r[0].summary.contains("DivX"));
    }

    #[test]
    fn parses_zypper_apk_xbps_outputs() {
        let z = parse_zypper_search("S | Name | Summary | Type\n--+------+---------+--------\n  | vlc  | Graphical media player | package\n  | vlc  | vlc | srcpackage\n");
        assert_eq!(z.len(), 1);
        let a = parse_apk_search("vlc-3.0.20-r7 - Multi-platform MPEG, VCD/DVD, and DivX player\n");
        assert_eq!((a[0].id.as_str(), a[0].version.as_str()), ("vlc", "3.0.20-r7"));
        let x = parse_xbps_search("[-] vlc-3.0.20_5          Multimedia player\n[*] vlc-devel-3.0.20_5    dev files\n");
        assert_eq!(x.len(), 2);
        assert_eq!(x[0].id, "vlc");
    }

    #[test]
    fn parses_flatpak_and_snap_outputs() {
        let f = parse_flatpak_search("org.videolan.VLC\tVLC\tVLC media player\t3.0.21\n");
        assert_eq!((f[0].id.as_str(), f[0].name.as_str()), ("org.videolan.VLC", "VLC"));
        let s = parse_snap_find("Name  Version  Publisher   Notes  Summary\nvlc   3.0.20   videolan✓   -      The ultimate media player\n");
        assert_eq!(s.len(), 1);
        assert_eq!((s[0].id.as_str(), s[0].summary.as_str()), ("vlc", "The ultimate media player"));
    }

    #[test]
    fn ranks_exact_matches_first() {
        let mut r = vec![
            result("native", Some("apt"), "firefox-l10n-fr", "firefox-l10n-fr", "", ""),
            result("flatpak", None, "org.mozilla.firefox", "Firefox", "", ""),
            result("native", Some("apt"), "webext-ublock-origin-firefox", "webext-ublock-origin-firefox", "", ""),
        ];
        rank(&mut r, "firefox");
        assert_eq!(r[0].id, "org.mozilla.firefox");
        assert_eq!(r[2].id, "webext-ublock-origin-firefox");
    }

    #[test]
    fn real_apt_search_works_on_this_machine_when_apt_exists() {
        if !binary_exists("apt-cache") {
            return;
        }
        let r = native_search("apt", "coreutils").expect("apt-cache search should run");
        assert!(r.iter().any(|x| x.id == "coreutils"), "{r:?}");
    }
}
