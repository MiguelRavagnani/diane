use bytes::Buf;
use chrono::{Local, NaiveDate, NaiveTime};
use serde::de::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

use crate::config::Config;
use crate::entry::Record;

const JOURNAL_PATH: &str = r"journal";
const ARCHIVE_PATH: &str = r"archive";

pub struct Vault {
    pub journal_path: PathBuf,
    pub archive_path: PathBuf,
    reference_day: NaiveDate,
}

impl Vault {
    pub fn new(config: &Config) -> Vault {
        Self {
            journal_path: Path::new(&config.diane_root).join(JOURNAL_PATH),
            archive_path: Path::new(&config.diane_root).join(ARCHIVE_PATH),
            reference_day: Local::now().date_naive(),
        }
    }

    pub fn append_journal_records(&self, records: &mut Vec<Record>) -> io::Result<()> {
        if records.is_empty() {
            return Ok(());
        }

        let reference_date = self.reference_day.format("%Y-%m-%d").to_string();
        let path = Path::new(&self.journal_path).join(format!("{}{}", reference_date, r".md"));

        fs::create_dir_all(&self.journal_path)?;

        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;

        let mut out = String::new();
        if file.metadata()?.len() == 0 {
            out.push_str(&format!("# {reference_date}\n\n"));
        }
        for record in records.iter() {
            let at = record.at.format("%H:%M");
            out.push_str(&format!("- {at} {}\n", record.text.trim()));
        }

        file.write_all(out.as_bytes())?;
        file.sync_all()?;

        records.clear();
        Ok(())
    }
}
