use std::process::Command;
use std::net::{UdpSocket};
use ureq;
use serde_json::Value;

pub struct OverviewData {
    pub hostname: String,
    pub ip: String,
    pub patroni_data: PatroniData,
    pub statuses: Vec<(String, String)>, // (name, "UP"/"DOWN")
    pub errors: Vec<(String, usize)>,    // (name, count)
}

pub struct PatroniData {
    pub role: String,
    pub leader: String,
    pub state: String,
    pub scope: String,
}

pub fn get_overview() -> OverviewData {
    let hostname = get_hostname();
    let ip = get_local_ip();
    let patroni_data = get_patroni_info();
    let statuses = check_services(&["patroni", "haproxy", "pgbouncer", "keepalived"]);
    let errors = count_errors(&["patroni", "haproxy", "pgbouncer", "keepalived"]);

    OverviewData {
        hostname,
        ip,
        patroni_data,
        statuses,
        errors,
    }
}

fn get_hostname() -> String {
    hostname::get().unwrap_or_default().to_string_lossy().to_string()
}

fn get_local_ip() -> String {
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|sock| {
            sock.connect("8.8.8.8:80")?;
            sock.local_addr()
        })
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|_| "unknown".into())
}

fn get_patroni_info() -> PatroniData {
    let url = "http://127.0.0.1:8008";
    
    match ureq::get(url).call() {
        Ok(resp) => {
            if let Ok(json) = resp.into_json::<Value>() {
                return PatroniData{
                    role: json["role"].as_str().unwrap_or("-").to_string(),
                    leader: json["leader"].as_str().unwrap_or("-").to_string(),
                    state: json["state"].as_str().unwrap_or("-").to_string(),
                    scope: json["patroni"]["scope"].as_str().unwrap_or("-").to_string(),
                };
            }
        }
        Err(ureq::Error::Status(503, resp)) => {
            if let Ok(json) = resp.into_json::<Value>() {
                return PatroniData{
                    role: json["role"].as_str().unwrap_or("-").to_string(),
                    leader: json["leader"].as_str().unwrap_or("-").to_string(),
                    state: json["state"].as_str().unwrap_or("-").to_string(),
                    scope: json["patroni"]["scope"].as_str().unwrap_or("-").to_string(),
                };
            }
        }
        Err(_) => {}
    }

    PatroniData {
        role: "-".to_string(),
        leader: "-".to_string(),
        state: "-".to_string(),
        scope: "-".to_string(),
    }
}

fn check_services(names: &[&str]) -> Vec<(String, String)> {
    names.iter().map(|&s| {
        let out = Command::new("systemctl")
            .arg("is-active")
            .arg(format!("{}.service", s))
            .output();

        let status = match out {
            Ok(output) if output.status.success() => "UP",
            _ => "DOWN",
        };
        (s.to_string(), status.to_string())
    }).collect()
}

fn count_errors(names: &[&str]) -> Vec<(String, usize)> {
    names.iter().map(|&s| {
        let out = Command::new("journalctl")
            .args(["-u", s, "-n", "300", "--no-pager"])
            .output();

        let count = match out {
            Ok(output) => {
                let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
                text.lines()
                    .filter(|line| line.contains("error") || line.contains("warn"))
                    .count()
            }
            _ => 0,
        };
        (s.to_string(), count)
    }).collect()
}