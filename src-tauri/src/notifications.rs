//! Desktop notifications, for alerts the user should see when NiTruX is
//! not the focused window.
//!
//! The dashboard already knows the CPU/RAM/disk thresholds the user set in
//! Preferences, but it could only colour a tile: crossing a threshold while
//! the window sat behind a browser produced nothing at all.

use crate::subprocess;
use std::time::Duration;

/// Both fields end up as `notify-send` arguments. Control characters are
/// stripped rather than escaped: a newline in a summary splits the
/// notification, and there is no legitimate reason for one to reach here.
fn sanitize(text: &str, max_len: usize) -> String {
    // Control characters become spaces rather than vanishing: dropping them
    // would weld words together ("CPU\nà 95 %" -> "CPUà 95 %").
    let spaced: String = text
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .take(max_len)
        .collect();
    spaced.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn validate_urgency(urgency: &str) -> Result<(), String> {
    match urgency {
        "low" | "normal" | "critical" => Ok(()),
        other => Err(format!("urgence inconnue : {other}")),
    }
}

#[tauri::command]
pub fn send_desktop_notification(summary: String, body: String, urgency: String) -> Result<(), String> {
    validate_urgency(&urgency)?;
    let summary = sanitize(&summary, 120);
    if summary.is_empty() {
        return Err("notification sans titre".to_string());
    }
    let body = sanitize(&body, 400);
    subprocess::run_with_timeout(
        "notify-send",
        &[
            "--app-name=NiTruX",
            "--urgency",
            urgency.as_str(),
            // `--` so a summary starting with a dash cannot become a flag.
            "--",
            summary.as_str(),
            body.as_str(),
        ],
        Duration::from_secs(10),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_control_characters_that_would_split_a_notification() {
        assert_eq!(sanitize("CPU\n\tà 95 %", 120), "CPU à 95 %");
    }

    #[test]
    fn truncates_instead_of_refusing_a_long_body() {
        let long = "a".repeat(1000);
        assert_eq!(sanitize(&long, 400).len(), 400);
    }

    #[test]
    fn refuses_an_urgency_outside_the_three_the_spec_defines() {
        assert!(validate_urgency("critical").is_ok());
        assert!(validate_urgency("urgent").is_err());
        assert!(validate_urgency("--help").is_err());
    }

    #[test]
    fn refuses_an_empty_summary() {
        let err = send_desktop_notification("   ".to_string(), String::new(), "normal".to_string())
            .expect_err("a notification with no title is not worth sending");
        assert!(err.contains("sans titre"), "{err}");
    }
}
