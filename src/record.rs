use std::time::SystemTime;

use Severity;

#[derive(Debug)]
pub struct Record<'a> {
    severity: Severity,
    timestamp: SystemTime,
    message: &'a str,
}

impl<'a> Record<'a> {
    pub fn new(severity: isize, message: &'a str) -> Record<'a> {
        Record {
            severity: severity,
            timestamp: SystemTime::now(),
            message: message,
        }
    }

    pub fn severity(&self) -> Severity {
        self.severity
    }

    pub fn message(&self) -> &str {
        self.message
    }
}
