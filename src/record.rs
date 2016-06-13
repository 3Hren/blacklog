use chrono::{DateTime, UTC};

use Severity;

#[derive(Debug)]
pub struct RecordBuilder<'a> {
    severity: Severity,
    message: &'a str,
}

impl<'a> RecordBuilder<'a> {
    pub fn severity(&self) -> Severity {
        self.severity
    }

    pub fn message(&self) -> &str {
        self.message
    }

    pub fn activate(self) -> Record<'a> {
        Record {
            record: self,
            timestamp: UTC::now(),
        }
    }
}

#[derive(Debug)]
pub struct Record<'a> {
    record: RecordBuilder<'a>,
    timestamp: DateTime<UTC>,
}

impl<'a> Record<'a> {
    pub fn new(severity: Severity, message: &'a str) -> RecordBuilder<'a> {
        RecordBuilder {
            severity: severity,
            message: message,
        }
    }

    pub fn severity(&self) -> Severity {
        self.record.severity
    }

    pub fn message(&self) -> &str {
        self.record.message
    }

    pub fn timestamp(&self) -> &DateTime<UTC> {
        &self.timestamp
    }
}
