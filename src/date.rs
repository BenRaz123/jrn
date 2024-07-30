//! module for the [`Date`] type. Literally only for timestamps. Most code for ser/de[^1] logic.
//! [^1]: Serialization/Deserialization

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

#[derive(Debug)]
/// The errors [`Date::from_str`] can return
pub enum DateFromStrError {
    /// Does not have two hyphen-minus ('-') characters
    InvalidLength,
    /// Quantities are not numeric
    IsNotNumeric,
}

impl FromStr for Date {
    type Err = DateFromStrError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items = s.split('-').collect::<Vec<&str>>();
        if items.len() != 3 {
            return Err(DateFromStrError::InvalidLength);
        }

        let year_result = items[0].parse::<i32>();
        let month_result = items[1].parse::<u8>();
        let day_result = items[2].parse::<u8>();

        match (year_result, month_result, day_result) {
            (Ok(year), Ok(month), Ok(day)) => Ok(Self { year, month, day }),
            _ => Err(DateFromStrError::IsNotNumeric),
        }
    }
}
