use std::time::SystemTime;

#[derive(Debug)]
pub struct Record<'a> {
    severity: i16,
    timestamp: SystemTime,
    message: &'a str,
}

impl<'a> Record<'a> {
    pub fn new(severity: i16, message: &'a str) -> Record<'a> {
        Record {
            severity: severity,
            timestamp: SystemTime::now(),
            message: message,
        }
    }

    pub fn message(&self) -> &str {
        self.message
    }
}
