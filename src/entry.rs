use chrono::{NaiveDate, NaiveTime};

pub struct Note {
    pub title: String,
    pub body: String,
    pub frontmatter: Vec<(String, String)>,
}

pub struct Record {
    pub at: NaiveTime,
    pub text: String,
}

pub struct Day {
    pub date: NaiveDate,
    pub records: Vec<Record>,
}
