use std::process::Command;

pub struct Logs {}

impl Logs {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read_logs(&self, service: &str, lines: usize) -> Vec<String> {
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
}
