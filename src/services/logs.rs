use crate::system;

pub struct LogsService {}

impl LogsService {
    pub fn new() -> Self {
        Self {}
    }

    /// Read logs for a specific service
    pub fn read_logs(&self, service: &str, lines: usize) -> Vec<String> {
        system::read_service_logs(service, lines)
    }
}