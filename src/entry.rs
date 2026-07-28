use chrono::{Local, NaiveDate, NaiveTime};

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

pub struct Day {
    pub date: NaiveDate,
    pub records: Vec<Record>,
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
}
