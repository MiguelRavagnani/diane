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
            return Err(format!("Missing title."));
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
