use crate::system;

pub struct Logs {}

impl Logs {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read_logs(&self, service: &str, lines: usize) -> Vec<String> {
        system::read_service_logs(service, lines)
    }
}
