//! Power profiles through power-profiles-daemon (`powerprofilesctl`): no
//! root and no NiTruX pkexec action -- the daemon's own polkit policy lets
//! the active session switch profiles, and the change is undone the same
//! way.

use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

const PROFILES: [&str; 3] = ["power-saver", "balanced", "performance"];

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct PowerProfiles {
    pub active: String,
    pub available: Vec<String>,
}

/// `powerprofilesctl list`: one "  name:" header per profile, the active
/// one prefixed with "* ".
fn parse_list(output: &str) -> PowerProfiles {
    let mut active = String::new();
    let mut available = Vec::new();
    for line in output.lines() {
        let Some(name) = line.trim().trim_start_matches("* ").strip_suffix(':') else { continue };
        if !PROFILES.contains(&name) {
            continue;
        }
        if line.trim_start().starts_with('*') {
            active = name.to_string();
        }
        available.push(name.to_string());
    }
    PowerProfiles { active, available }
}

#[tauri::command]
pub fn get_power_profiles() -> Result<PowerProfiles, String> {
    let out = subprocess::run_with_timeout("powerprofilesctl", &["list"], Duration::from_secs(5))?;
    Ok(parse_list(&out))
}

#[tauri::command]
pub fn set_power_profile(profile: String) -> Result<(), String> {
    if !PROFILES.contains(&profile.as_str()) {
        return Err(format!("profil inconnu : {profile}"));
    }
    subprocess::run_with_timeout("powerprofilesctl", &["set", &profile], Duration::from_secs(10)).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_real_list_output() {
        // Captured on Omarchy (power-profiles-daemon 0.30).
        let out = "  performance:\n    CpuDriver:\tintel_pstate\n    Degraded:   no\n\n* balanced:\n    CpuDriver:\tintel_pstate\n    PlatformDriver:\tplaceholder\n\n  power-saver:\n    CpuDriver:\tintel_pstate\n";
        let p = parse_list(out);
        assert_eq!(p.active, "balanced");
        assert_eq!(p.available, vec!["performance", "balanced", "power-saver"]);
    }

    #[test]
    fn refuses_anything_but_the_three_profiles() {
        assert!(set_power_profile("--help".into()).is_err());
        assert!(set_power_profile("turbo".into()).is_err());
    }
}
