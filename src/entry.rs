use chrono::{Local, NaiveDate, NaiveTime};
use regex::Regex;

pub struct Note {
    pub slug: String,
    pub title: String,
    pub created: NaiveDate,
    pub updated: NaiveDate,
    pub body: String,
}

impl Note {
    pub fn new(title: String, body: String, created: Option<NaiveDate>) -> Result<Note, String> {
        if title.is_empty() {
            return Err("Missing title.".to_string());
        }

        let now = Local::now().date_naive();

        let created_at = match created {
            Some(date) => date,
            None => now,
        };

        Ok(Self {
            slug: slugify(title.as_str()),
            title,
            created: created_at,
            updated: now,
            body,
        })
    }

    pub fn is_valid(&self) -> bool {
        if self.slug.is_empty() || self.title.is_empty() {
            return false;
        }

        true
    }

    pub fn to_text(&self) -> String {
        format!(
            "---\ntitle: {}\ncreated: {}\nupdated: {}\ntags: todo\n---\n\n{}",
            self.title,
            self.created.format("%Y-%m-%d").to_string().to_owned(),
            self.updated.format("%Y-%m-%d").to_string().to_owned(),
            self.body
        )
    }
}

impl TryFrom<String> for Note {
    type Error = &'static str;

    fn try_from(string: String) -> Result<Self, Self::Error> {
        let rest = string.strip_prefix("---\n").ok_or("missing formatter")?;
        let (front, body) = rest.split_once("\n---\n").ok_or("unterminated formatter")?;

        let mut title = None;
        let mut created = None;
        let mut updated = None;

        for line in front.lines() {
            let Some((key, value)) = line.split_once(": ") else {
                continue;
            };
            match key {
                "title" => title = Some(value.to_string()),
                "created" => {
                    created = Some(
                        NaiveDate::parse_from_str(value, "%Y-%m-%d")
                            .map_err(|_| "bad created date")?,
                    )
                }
                "updated" => {
                    updated = Some(
                        NaiveDate::parse_from_str(value, "%Y-%m-%d")
                            .map_err(|_| "bad updated date")?,
                    )
                }
                _ => {}
            }
        }

        let title = title.ok_or("missing title")?;
        let created = created.ok_or("missing created")?;
        let updated = updated.ok_or("missing updated")?;
        let body = body.to_owned();

        Ok(Note {
            slug: slugify(&title),
            title,
            created,
            updated,
            body,
        })
    }
}

fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub struct Record {
    pub at: NaiveTime,
    pub text: String,
}

const RECORD_REGEX: &str = r"^-\s*(?P<time>\d{2}:\d{2})\s*(?P<content>.*)$";

impl TryFrom<String> for Record {
    // TODO: Need for an aggregated error type strategy is growing
    type Error = &'static str;

    fn try_from(string: String) -> Result<Self, Self::Error> {
        let Ok(re) = Regex::new(RECORD_REGEX) else {
            return Err("Invalid record regex");
        };

        if let Some(record_caps) = re.captures(&string) {
            let time_str = &record_caps["time"];
            let text = record_caps["content"].trim().to_string();

            if let Ok(at) = NaiveTime::parse_from_str(time_str, "%H:%M") {
                return Ok(Record { at, text });
            }
        }

        Err("Unable to convert string to record")
    }
}

#[derive(Default)]
pub struct Day {
    pub records: Vec<Record>,
    // dirty: bool, // TODO: Selective flushing needed :)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Note {
        Note::new("Kafka setup".into(), "body".into(), None).unwrap()
    }

    #[test]
    fn new_derives_slug_from_title() {
        assert_eq!(sample().slug, "kafka-setup");
    }

    #[test]
    fn slug_lowercases_and_hyphenates() {
        assert_eq!(slugify("DSP / FFT notes"), "dsp-fft-notes");
    }

    #[test]
    fn new_uses_given_created_date() {
        let d = NaiveDate::from_ymd_opt(2020, 1, 2).unwrap();
        let note = Note::new("x".into(), String::new(), Some(d)).unwrap();
        assert_eq!(note.created, d);
    }

    #[test]
    fn note_roundtrips_through_text() {
        let note = sample();
        let back = Note::try_from(note.to_text()).unwrap();
        assert_eq!(back.title, note.title);
        assert_eq!(back.slug, note.slug);
        assert_eq!(back.created, note.created);
        assert_eq!(back.updated, note.updated);
        assert_eq!(back.body.trim(), note.body);
    }

    #[test]
    fn record_parses_time_and_text() {
        let r = Record::try_from("-  09:30   shipped the parser  ".to_string()).unwrap();
        assert_eq!(r.at, NaiveTime::from_hms_opt(9, 30, 0).unwrap());
        assert_eq!(r.text, "shipped the parser");
    }

    #[test]
    fn record_rejects_non_record_lines() {
        assert!(Record::try_from("# 2026-08-29".to_string()).is_err());
        assert!(Record::try_from("- 25:00 bad hour".to_string()).is_err());
    }
}
