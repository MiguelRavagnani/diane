use bytes::Buf;
use chrono::{Local, NaiveDate, NaiveTime};
use serde::de::Error;
use std::fs::{self, File};
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
    pub fn new(&self, config: &Config) -> Vault {
        Self {
            journal_path: Path::new(&config.diane_root).join(JOURNAL_PATH),
            archive_path: Path::new(&config.diane_root).join(ARCHIVE_PATH),
            reference_day: Local::now().date_naive(),
        }
    }

    pub fn append_journal_records(&self, records: &mut Vec<Record>) -> io::Result<()> {
        let reference_date = self.reference_day.format("%Y-%m-%d").to_string();
        let path = Path::new(&self.journal_path).join(format!("{}{}", reference_date, r".md"));

        let mut temp_file = NamedTempFile::new_in(self.journal_path.clone())?;
        let mut record_stream = "".to_owned();

        for mut record in records.drain(..) {
            let at = record.at.format("%H:%M").to_string();
            record.text.insert_str(0, &format!("\n- {at}"));

            if path.exists() {
                let mut existing_file = File::open(&path)?;
                io::copy(&mut existing_file, &mut temp_file)?;
                record.text.insert_str(0, &format!("# {reference_date}"));
            }

            record_stream += &record.text;
        }

        temp_file.write_all(&record_stream.into_bytes())?;
        temp_file.flush()?;

        temp_file.persist(path)?;

        Ok(())
    }
}
