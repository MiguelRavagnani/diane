use chrono::{Local, NaiveDate};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

use crate::config::Config;
use crate::entry::{Day, Note, Record};

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

    pub fn recover_journal_records(&self) -> io::Result<Day> {
        // TODO: Propper error return here. might not want an error if empty
        let journal_entries = fs::read_dir(&self.journal_path)?;
        let mut days: Vec<Day> = Vec::new();

        for journal_entry in journal_entries {
            let journal_entry = journal_entry?;
            let journal_entry_path = journal_entry.path();

            if journal_entry_path.is_file()
                && let Some(journal_entry_filename) =
                    journal_entry_path.file_stem().and_then(|s| s.to_str())
            {
                match NaiveDate::parse_from_str(journal_entry_filename, "%Y-%m-%d") {
                    Ok(date) => {
                        let mut records: Vec<Record> = Vec::new();
                        let entry_records_content = File::open(&journal_entry_path)?;
                        let reader = BufReader::new(entry_records_content);

                        for entry_record_row in reader.lines() {
                            let entry_record_row = entry_record_row?;
                            if let Ok(record) = Record::try_from(entry_record_row) {
                                records.push(record);
                            }
                        }

                        days.push(Day { date, records })
                    }
                    Err(_) => todo!(),
                }
            }
        }

        let reference_date = self.reference_day.format("%Y-%m-%d").to_string();
        let path = Path::new(&self.journal_path).join(format!("{}.md", reference_date));
        todo!()
    }

    pub fn append_journal_records(&self, records: &mut Vec<Record>) -> io::Result<()> {
        if records.is_empty() {
            return Ok(());
        }

        let reference_date = self.reference_day.format("%Y-%m-%d").to_string();
        let path = Path::new(&self.journal_path).join(format!("{}.md", reference_date));

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

    pub fn save_archive_note(&self, note: &Note) -> io::Result<()> {
        if !note.is_valid() {
            return Ok(());
        }

        let path = Path::new(&self.archive_path).join(format!("{}.md", note.slug));

        fs::create_dir_all(&self.archive_path)?;

        let mut tmp = NamedTempFile::new_in(&self.archive_path)?;
        tmp.write_all(note.to_text().as_bytes())?;
        tmp.flush()?;
        tmp.persist(&path)?;

        Ok(())
    }
}
