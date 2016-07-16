use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Error, Write};
use std::path::{Path, PathBuf};
use std::str;
use std::sync::{Arc, Mutex};

use factory::Factory;
use layout::Layout;
use layout::pattern::{ParseError, PatternLayout};
use output::Output;
use registry::{Config, Registry};
use record::Record;

/// Writes all messages into one or multiple files.
///
/// # Note
///
/// Double locking strategy was chosen to enable concurrent writing into different files from
/// multiple threads.
pub struct FileOutput {
    pattern: PatternLayout,
    // TODO: Replace `File` with `Writer` and add flushing policies.
    files: Mutex<HashMap<PathBuf, Arc<Mutex<BufWriter<File>>>>>,
}

impl FileOutput {
    pub fn new(pattern: &str) -> Result<FileOutput, ParseError> {
        let pattern = PatternLayout::new(pattern)?;

        let res = FileOutput {
            pattern: pattern,
            files: Mutex::new(HashMap::new()),
        };

        Ok(res)
    }
}

impl Output for FileOutput {
    fn write(&self, rec: &Record, message: &[u8]) -> Result<(), Error> {
        let mut buf = Vec::new();
        self.pattern.format(rec, &mut buf).unwrap();

        let path = str::from_utf8(&buf).unwrap();
        let path = Path::new(path);

        let file = {
            let mut files = self.files.lock().unwrap();

            // TODO: Not optimal, because of heap allocation every try.
            match files.entry(path.to_path_buf()) {
                Entry::Occupied(v) => v.get().clone(),
                Entry::Vacant(v) => {
                    let file = OpenOptions::new().append(true).create(true).open(path)?;
                    v.insert(Arc::new(Mutex::new(BufWriter::new(file)))).clone()
                }
            }
        };

        let mut file = file.lock().unwrap();
        file.write_all(message)?;
        file.write_all(b"\n")
    }
}

impl Factory for FileOutput {
    type Item = Output;

    fn ty() -> &'static str {
        "file"
    }

    fn from(cfg: &Config, _registry: &Registry) -> Result<Box<Output>, Box<error::Error>> {
        let path = cfg.find("path")
            .ok_or("field \"path\" is required")?
            .as_string()
            .ok_or("field \"path\" must be a string")?;

        let res = FileOutput::new(path)?;

        Ok(box res)
    }
}
