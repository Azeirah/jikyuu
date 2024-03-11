use serde::{Deserialize, Serialize};
use std::str::FromStr;

use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, Weekday};

use crate::error;

#[derive(Debug)]
pub enum CommitTimeBound {
    Always,
    Today,
    Yesterday,
    ThisWeek,
    LastWeek,
    Date(NaiveDate),
}

impl CommitTimeBound {
    pub fn to_date_time(&self) -> Option<NaiveDateTime> {
        let zero = || NaiveTime::from_hms(0, 0, 0);

        match self {
            Self::Always => None,
            Self::Today => {
                let local = Local::today();
                let date = NaiveDate::from_ymd(local.year(), local.month(), local.day());
                Some(NaiveDateTime::new(date, zero()))
            }
            Self::Yesterday => {
                let local = Local::today();
                let date = NaiveDate::from_ymd(local.year(), local.month(), local.day())
                    - Duration::days(1);
                Some(NaiveDateTime::new(date, zero()))
            }
            Self::ThisWeek => {
                let local = Local::today();
                let date =
                    NaiveDate::from_isoywd(local.year(), local.iso_week().week(), Weekday::Sun);
                Some(NaiveDateTime::new(date, zero()))
            }
            Self::LastWeek => {
                let local = Local::today();
                let date =
                    NaiveDate::from_isoywd(local.year(), local.iso_week().week(), Weekday::Sun)
                        - Duration::weeks(1);
                Some(NaiveDateTime::new(date, zero()))
            }
            Self::Date(date) => Some(NaiveDateTime::new(date.clone(), zero())),
        }
    }
}

impl FromStr for CommitTimeBound {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "always" => Ok(Self::Always),
            "today" => Ok(Self::Today),
            "yesterday" => Ok(Self::Yesterday),
            "thisweek" => Ok(Self::ThisWeek),
            "lastweek" => Ok(Self::LastWeek),
            x => match NaiveDate::from_str(x) {
                Ok(date) => Ok(Self::Date(date)),
                Err(_) => Err(error::Error::new(format!(
                    "Could not parse date '{}' using YYYY-mm-dd format",
                    x
                ))),
            },
        }
    }
}

impl ToString for CommitTimeBound {
    fn to_string(&self) -> String {
        match self {
            Self::Always => "always".into(),
            Self::Today => "today".into(),
            Self::Yesterday => "yesterday".into(),
            Self::ThisWeek => "thisweek".into(),
            Self::LastWeek => "lastweek".into(),
            Self::Date(date) => date.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CommitHours {
    pub email: Option<String>,
    pub author_name: Option<String>,
    pub duration: Duration,
    pub commit_count: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CommitHoursJson {
    pub email: Option<String>,
    pub author_name: Option<String>,
    pub hours: f32,
    pub commit_count: usize,
}

impl From<&CommitHours> for CommitHoursJson {
    fn from(time: &CommitHours) -> Self {
        CommitHoursJson {
            email: time.email.clone(),
            author_name: time.author_name.clone(),
            hours: time.duration.num_minutes() as f32 / 60.0,
            commit_count: time.commit_count,
        }
    }
}
