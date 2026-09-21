//! Read-only Docker container/image listing, shelling out to the `docker`
//! CLI via `subprocess::run_with_timeout` and parsing its `{{json .}}`
//! JSON-lines output — the same parsing pattern as `logs.rs` (journalctl).
//!
//! Infallible by design, same rationale as `network::get_network_snapshot`:
//! Docker not being installed is a normal, common case (not every NiTruX
//! user runs Docker), reflected via `available: false` rather than an error
//! the frontend has to specifically handle.

use crate::subprocess;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct Container {
    pub id: String,
    pub image: String,
    pub name: String,
    pub status: String,
}

#[derive(Serialize, Clone)]
pub struct DockerImage {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size: String,
}

#[derive(Serialize, Clone)]
pub struct DockerSnapshot {
    pub available: bool,
    /// Whether the `docker` binary itself is present on `PATH`. Distinct
    /// from `available`: a host can have Docker installed but its daemon
    /// stopped, or the invoking user missing from the `docker` group --
    /// both produce `available: false` from a failed `docker ps`, but are
    /// a materially different, actionable situation from "not installed
    /// at all" and deserve a different message to the user.
    pub installed: bool,
    /// The real error from the failed `docker ps` call (e.g. "daemon is
    /// running?" / "permission denied"), when `installed` is true but
    /// `available` is false. `None` whenever Docker isn't installed at
    /// all, or the call succeeded.
    pub error: Option<String>,
    pub containers: Vec<Container>,
    pub images: Vec<DockerImage>,
}

#[derive(Deserialize)]
struct RawContainer {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Image")]
    image: String,
    #[serde(rename = "Names")]
    names: String,
    #[serde(rename = "Status")]
    status: String,
}

#[derive(Deserialize)]
struct RawImage {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Repository")]
    repository: String,
    #[serde(rename = "Tag")]
    tag: String,
    #[serde(rename = "Size")]
    size: String,
}

pub fn parse_container_line(line: &str) -> Option<Container> {
    let raw: RawContainer = serde_json::from_str(line).ok()?;
    Some(Container {
        id: raw.id,
        image: raw.image,
        name: raw.names,
        status: raw.status,
    })
}

pub fn parse_image_line(line: &str) -> Option<DockerImage> {
    let raw: RawImage = serde_json::from_str(line).ok()?;
    Some(DockerImage {
        id: raw.id,
        repository: raw.repository,
        tag: raw.tag,
        size: raw.size,
    })
}

/// Infallible by design, same rationale as `network::get_network_snapshot`:
/// Docker not being installed is a normal, common case (not every NiTruX
/// user runs Docker), reflected via `available: false` rather than an error
/// the frontend has to specifically handle.
#[tauri::command]
pub fn get_docker_snapshot() -> DockerSnapshot {
    let installed = crate::packages::binary_exists("docker");

    let containers_result = subprocess::run_with_timeout(
        "docker",
        &["ps", "-a", "--format", "{{json .}}"],
        Duration::from_secs(10),
    );
    let available = containers_result.is_ok();
    let error = containers_result.as_ref().err().cloned();

    let containers = containers_result
        .map(|output| output.lines().filter_map(parse_container_line).collect())
        .unwrap_or_default();

    let images = subprocess::run_with_timeout(
        "docker",
        &["images", "--format", "{{json .}}"],
        Duration::from_secs(10),
    )
    .map(|output| output.lines().filter_map(parse_image_line).collect())
    .unwrap_or_default();

    DockerSnapshot {
        available,
        installed,
        error,
        containers,
        images,
    }
}

/// Docker ids are hex digests (or the short prefix of one) and image ids
/// may carry a `sha256:` prefix. Anything else is refused rather than
/// forwarded to `docker`, so a value coming from the frontend can never
/// turn into an extra argument or a flag.
pub fn validate_docker_id(id: &str) -> Result<(), String> {
    let bare = id.strip_prefix("sha256:").unwrap_or(id);
    if bare.is_empty() || bare.len() > 71 {
        return Err("identifiant Docker invalide".to_string());
    }
    if !bare.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("identifiant Docker invalide : {id}"));
    }
    Ok(())
}

fn validate_container_action(action: &str) -> Result<(), String> {
    match action {
        "start" | "stop" | "restart" => Ok(()),
        other => Err(format!("action de conteneur non autorisée : {other}")),
    }
}

/// Starts, stops or restarts one container. Runs unprivileged on purpose --
/// Docker access is granted by the `docker` group (or a rootless daemon),
/// never by NiTruX elevating itself: adding a pkexec path here would hand
/// root to anything the daemon can run.
#[tauri::command]
pub fn docker_container_action(container_id: String, action: String) -> Result<String, String> {
    validate_docker_id(&container_id)?;
    validate_container_action(&action)?;
    subprocess::run_with_timeout(
        "docker",
        &[action.as_str(), container_id.as_str()],
        Duration::from_secs(30),
    )
}

#[tauri::command]
pub fn docker_container_logs(container_id: String) -> Result<String, String> {
    validate_docker_id(&container_id)?;
    // `docker logs` writes application output to stderr as often as to
    // stdout, so both streams are kept and concatenated in order.
    let (stdout, stderr, code) = subprocess::run_capturing_exit_code(
        "docker",
        &["logs", "--tail", "200", container_id.as_str()],
        Duration::from_secs(20),
    )?;
    if code != 0 {
        return Err(format!("docker logs a échoué (code {code}) : {}", stderr.trim()));
    }
    Ok(format!("{stdout}{stderr}"))
}

#[tauri::command]
pub fn docker_image_remove(image_id: String) -> Result<String, String> {
    validate_docker_id(&image_id)?;
    subprocess::run_with_timeout("docker", &["rmi", image_id.as_str()], Duration::from_secs(60))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_real_container_id_and_a_sha256_image_id() {
        assert!(validate_docker_id("a1b2c3d4e5f6").is_ok());
        assert!(validate_docker_id("sha256:9f2c1e0b").is_ok());
    }

    #[test]
    fn refuses_anything_that_is_not_a_hex_id() {
        // The exact shapes that would matter if they reached `docker`:
        // a flag, a second argument, a shell metacharacter.
        for bad in ["--rm", "web container", "a1b2; rm -rf /", "", "nginx:latest"] {
            assert!(validate_docker_id(bad).is_err(), "{bad} should be refused");
        }
    }

    #[test]
    fn refuses_a_container_action_outside_the_whitelist() {
        assert!(validate_container_action("start").is_ok());
        assert!(validate_container_action("exec").is_err());
        assert!(validate_container_action("rm").is_err());
    }

    #[test]
    fn parses_docker_ps_json_line() {
        let line = r#"{"ID":"a1b2c3d4","Image":"nginx:latest","Names":"web","Status":"Up 2 hours"}"#;
        let c = parse_container_line(line).expect("should parse");
        assert_eq!(c.id, "a1b2c3d4");
        assert_eq!(c.image, "nginx:latest");
        assert_eq!(c.name, "web");
        assert_eq!(c.status, "Up 2 hours");
    }

    #[test]
    fn skips_unparseable_container_line() {
        assert!(parse_container_line("not json").is_none());
    }

    #[test]
    fn parses_docker_images_json_line() {
        let line = r#"{"ID":"e5f6a7b8","Repository":"nginx","Tag":"latest","Size":"142MB"}"#;
        let img = parse_image_line(line).expect("should parse");
        assert_eq!(img.id, "e5f6a7b8");
        assert_eq!(img.repository, "nginx");
        assert_eq!(img.tag, "latest");
        assert_eq!(img.size, "142MB");
    }

    #[test]
    fn skips_unparseable_image_line() {
        assert!(parse_image_line("not json").is_none());
    }
}
