use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct DriverSnapshot {
    pub loaded_modules: Vec<String>,
    pub gpu_driver: String,
}

pub fn parse_lsmod_line(line: &str) -> Option<String> {
    let name = line.split_whitespace().next()?;
    if name.eq_ignore_ascii_case("module") {
        return None; // header row
    }
    Some(name.to_string())
}

pub fn detect_gpu_driver(modules: &[String]) -> String {
    if modules.iter().any(|m| m == "nvidia") {
        "nvidia (propriétaire)".to_string()
    } else if modules.iter().any(|m| m == "nouveau") {
        "nouveau (open-source)".to_string()
    } else if modules.iter().any(|m| m == "amdgpu") {
        "amdgpu (open-source)".to_string()
    } else if modules.iter().any(|m| m == "i915") {
        "i915 (Intel, open-source)".to_string()
    } else {
        "inconnu".to_string()
    }
}

fn run_lsmod() -> Result<Vec<String>, String> {
    let output = subprocess::run_with_timeout("lsmod", &[], Duration::from_secs(5))?;
    Ok(output.lines().filter_map(parse_lsmod_line).collect())
}

#[tauri::command]
pub fn get_driver_snapshot() -> Result<DriverSnapshot, String> {
    let loaded_modules = run_lsmod()?;
    let gpu_driver = detect_gpu_driver(&loaded_modules);
    Ok(DriverSnapshot { loaded_modules, gpu_driver })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_nvidia_driver_from_module_list() {
        let modules = vec!["nvidia".to_string(), "snd_hda_intel".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "nvidia (propriétaire)");
    }

    #[test]
    fn detects_nouveau_driver_from_module_list() {
        let modules = vec!["nouveau".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "nouveau (open-source)");
    }

    #[test]
    fn detects_amdgpu_driver_from_module_list() {
        let modules = vec!["amdgpu".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "amdgpu (open-source)");
    }

    #[test]
    fn falls_back_to_unknown_when_no_gpu_module_present() {
        let modules = vec!["ext4".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "inconnu");
    }

    #[test]
    fn parses_lsmod_line_into_module_name() {
        let line = "nvidia               56655872  42";
        assert_eq!(parse_lsmod_line(line), Some("nvidia".to_string()));
    }
}
