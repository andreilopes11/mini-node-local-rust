use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AppendOnlyLog {
    path: PathBuf,
}

impl AppendOnlyLog {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn append_line(&self, line: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        writeln!(file, "{line}")?;
        file.sync_data()?;
        Ok(())
    }

    pub fn read_lines(&self) -> io::Result<Vec<String>> {
        match File::open(&self.path) {
            Ok(file) => {
                let reader = io::BufReader::new(file);
                reader.lines().collect()
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(error) => Err(error),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_log_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("mini-node-{name}-{nanos}.log"))
    }

    #[test]
    fn appends_and_reads_lines() {
        let path = temp_log_path("append");
        let log = AppendOnlyLog::new(&path);

        log.append_line("first").unwrap();
        log.append_line("second").unwrap();

        assert_eq!(log.read_lines().unwrap(), vec!["first", "second"]);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn missing_log_reads_as_empty() {
        let path = temp_log_path("missing");
        let log = AppendOnlyLog::new(path);

        assert!(log.read_lines().unwrap().is_empty());
    }
}
