//! module for the [`Date`] type. Literally only for timestamps. Most code for ser/de[^1] logic.
//! [^1]: Serialization/Deserialization

use chrono::DateTime;
use chrono::Local;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use enum_display::EnumDisplay;
use chrono::Datelike;
use serde::Deserialize;
use serde::Serialize;
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
/// A date with a year, month, and day value
pub struct Date {
    /// the year. can be negative for BC\[E\]
    pub year: i32,
    /// the month
    pub month: u8,
    /// the day
    pub day: u8,
}

impl Date {
    /// returns today's date
    pub fn today() -> Self {
        let current_date = chrono::Local::now();
        let year = current_date.year();
        let month = current_date.month() as u8;
        let day = current_date.day() as u8;
        Self { year, month, day }
    }
}

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::from_str(&s).map_err(|_| serde::de::Error::custom("Could not parse into timestamp"))
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Debug, EnumDisplay)]
/// The errors [`Date::from_str`] can return
pub enum DateFromStrError {
    InvalidTodayMinusFormat,
    /// Does not have two hyphen-minus ('-') characters
    InvalidLength,
    /// Quantities are not numeric
    IsNotNumeric,
    InvalidDate,
}

impl FromStr for Date {
    type Err = DateFromStrError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();

        if s.starts_with("today") {
            let rest = &s[5..].trim();
            if rest.is_empty() {
                return Ok(Self::today());
            }

            if !rest.starts_with("-") {
                return Err(DateFromStrError::InvalidTodayMinusFormat);
            }

            let minus_days = &rest[1..].trim().parse::<u128>();

            if minus_days.is_err() {
                return Err(DateFromStrError::InvalidTodayMinusFormat);
            }
            
            let delta = chrono::TimeDelta::try_days(minus_days.to_owned().unwrap() as i64);

            if delta.is_none() {
                return Err(DateFromStrError::InvalidDate);
            }

            let delta = delta.unwrap();

            let chrono_today: DateTime<Local> = Local::now();
        
            let chrono_with_delta = chrono_today.checked_sub_signed(delta);

            if chrono_with_delta.is_none() {
                return Err(DateFromStrError::InvalidDate);
            }

            let chrono_with_delta = chrono_with_delta.unwrap();

            return Ok(Self { year: chrono_with_delta.year(), month: chrono_with_delta.month() as u8, day: chrono_with_delta.day() as u8 })
        }

        let items = s.split('-').collect::<Vec<&str>>();
        if items.len() != 3 {
            return Err(DateFromStrError::InvalidLength);
        }

        let year_result = items[0].parse::<i32>();
        let month_result = items[1].parse::<u8>();
        let day_result = items[2].parse::<u8>();

        match (year_result, month_result, day_result) {
            (Ok(year), Ok(month), Ok(day)) => {
                let chrono_date = NaiveDate::from_ymd_opt(year, month as u32, day as u32);
                match chrono_date {
                    None => Err(DateFromStrError::InvalidDate),
                    Some(_) => Ok(Self {year, month, day})
                }
            },
            _ => Err(DateFromStrError::IsNotNumeric),
        }
    }
}
