use chrono::{DateTime, UTC};

use Severity;

enum Value<'a> {
    String(&'a str),
}

struct Meta<'a> {
    name: &'a str,
    value: Value<'a>,
}

struct MetaList<'a> {
    meta: &'a [Meta<'a>],
    next: Option<&'a MetaList<'a>>,
}

#[derive(Debug)]
pub struct RecordBuilder<'a> {
    severity: Severity,
    message: &'a str,
    // thread: usize,
    // args: Arguments<'a>,
    // meta: &'a MetaList<'a>,

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
