use serde::Serialize;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

const MAX_PORTS_PER_SCAN: usize = 200;
const CONNECT_TIMEOUT: Duration = Duration::from_millis(300);

#[derive(Serialize, Clone)]
pub struct PortResult {
    pub port: u16,
    pub open: bool,
}

/// Rejects unreasonably large scan requests up front, before any network
/// activity — this is what keeps the scanner "bounded" rather than a
/// potential DoS/abuse vector against the target host.
pub fn validate_port_list(ports: &[u16]) -> Result<(), String> {
    if ports.len() > MAX_PORTS_PER_SCAN {
        return Err(format!(
            "trop de ports demandés ({}), maximum {MAX_PORTS_PER_SCAN} par scan",
            ports.len()
        ));
    }
    Ok(())
}

/// Attempts a short TCP connect to each port in `ports` on `host`,
/// sequentially, each bounded by `CONNECT_TIMEOUT`. Does not distinguish
/// "host unreachable" from "port closed" — both report `open: false`,
/// which is the correct, safe default for a diagnostic tool (no need to
/// leak network topology details beyond open/closed on the requested host).
pub fn scan_ports(host: &str, ports: &[u16]) -> Vec<PortResult> {
    ports
        .iter()
        .map(|&port| {
            let open = format!("{host}:{port}")
                .to_socket_addrs()
                .ok()
                .and_then(|mut addrs| addrs.next())
                .map(|addr| TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT).is_ok())
                .unwrap_or(false);
            PortResult { port, open }
        })
        .collect()
}

#[tauri::command]
pub fn scan_ports_cmd(host: String, ports: Vec<u16>) -> Result<Vec<PortResult>, String> {
    validate_port_list(&ports)?;
    Ok(scan_ports(&host, &ports))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn detects_an_open_port_on_localhost() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("should bind");
        let port = listener.local_addr().unwrap().port();

        let results = scan_ports("127.0.0.1", &[port]);
        drop(listener);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].port, port);
        assert!(results[0].open);
    }

    #[test]
    fn reports_a_closed_port_as_not_open() {
        // Port 1 is a reserved/unassigned port extremely unlikely to have
        // anything listening on it in any test environment.
        let results = scan_ports("127.0.0.1", &[1]);
        assert_eq!(results.len(), 1);
        assert!(!results[0].open);
    }

    #[test]
    fn caps_the_port_list_to_a_safe_maximum() {
        let too_many: Vec<u16> = (1..=2000).collect();
        let err = validate_port_list(&too_many).expect_err("should reject");
        assert!(err.contains("maximum"));
    }

    #[test]
    fn accepts_a_reasonable_port_list() {
        assert!(validate_port_list(&[22, 80, 443]).is_ok());
    }
}
