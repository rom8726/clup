use std::process::Command;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;

/// Execute a system command and return its output as a String
pub fn exec_command(cmd: &str, args: &[&str]) -> io::Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Command failed with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ),
        ))
    }
}

/// Check if a systemd service is active
pub fn is_service_active(service: &str) -> bool {
    Command::new("systemctl")
        .args(["is-active", &format!("{service}.service")])
        .output()
        .map_or(false, |o| o.status.success())
}

/// Get service errors from journal
pub fn get_service_errors(service: &str, lines: u32) -> u32 {
    Command::new("journalctl")
        .args(["-u", service, "-n", &lines.to_string(), "--no-pager"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .to_lowercase()
                .lines()
                .filter(|l| l.contains("error") || l.contains("fatal"))
                .count() as u32
        })
        .unwrap_or(0)
}

/// Get service uptime
pub fn get_service_uptime(service: &str) -> String {
    Command::new("systemctl")
        .args([
            "show",
            &format!("{service}.service"),
            "--property=ActiveEnterTimestamp",
        ])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .split('=')
                .nth(1)
                .unwrap_or("unknown")
                .to_string()
        })
        .unwrap_or_else(|_| "unknown".into())
}

/// Detect service version
pub fn detect_service_version(service: &str) -> String {
    let try_cmd = |arg: &str| -> Option<String> {
        Command::new(service)
            .arg(arg)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(
                        String::from_utf8_lossy(&o.stdout)
                            .lines()
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string(),
                    )
                } else {
                    None
                }
            })
    };

    try_cmd("-v")
        .or_else(|| try_cmd("--version"))
        .unwrap_or_else(|| "-".into())
}

/// Read logs from journald
pub fn read_service_logs(service: &str, lines: usize) -> Vec<String> {
    let output = Command::new("journalctl")
        .args(["-u", service, "-n", &lines.to_string(), "--no-pager"])
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        stdout.lines().map(|s| s.to_string()).collect()
    } else {
        vec!["Failed to read logs".to_string()]
    }
}

/// Communicate with HAProxy via Unix socket
pub fn query_haproxy_socket(socket_path: &str, command: &str) -> io::Result<String> {
    let mut stream = UnixStream::connect(socket_path)?;
    stream.write_all(command.as_bytes())?;
    
    let mut buf = String::new();
    stream.read_to_string(&mut buf)?;
    
    Ok(buf)
}

/// Detect Keepalived VIP
pub fn detect_keepalived_vip() -> String {
    // -------- 1. Try through JSON output (`ip -j …`) ----------
    if let Ok(out) = Command::new("ip")
        .args(["-j", "-4", "addr", "show", "scope", "global"])
        .output()
    {
        if out.status.success() {
            if let Ok(ifaces) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                if let Some(arr) = ifaces.as_array() {
                    for iface in arr {
                        // addr_info ‒ array with IP infos
                        if let Some(addr_arr) = iface.get("addr_info").and_then(|v| v.as_array()) {
                            for addr in addr_arr {
                                let label = addr.get("label").and_then(|v| v.as_str()).unwrap_or("");
                                let flags = addr.get("flags").and_then(|v| v.as_array());

                                // Signs of VIP
                                let is_secondary = flags.map_or(false, |f| {
                                    f.iter().any(|fl| fl.as_str() == Some("secondary"))
                                });
                                let has_colon_in_label = label.contains(':');

                                if is_secondary || has_colon_in_label {
                                    if let Some(local) = addr.get("local").and_then(|v| v.as_str()) {
                                        return local.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Ok(out) = Command::new("ip")
        .args(["-o", "-4", "addr", "show", "scope", "global"])
        .output()
    {
        if out.status.success() {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                let has_secondary = line.contains(" secondary ");
                let has_label_colon = line.split_whitespace().any(|w| w.contains(':'));

                if has_secondary || has_label_colon {
                    if let Some(addr_field) = line.split_whitespace().nth(3) {
                        return addr_field
                            .split('/')
                            .next()
                            .unwrap_or("-")
                            .to_string();
                    }
                }
            }
        }
    }

    "-".into()
}