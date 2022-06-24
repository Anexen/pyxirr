use std::error::Error;
use std::fmt;
use time::{macros::format_description, Date};

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct DateLike(i32);

impl From<i32> for DateLike {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<Date> for DateLike {
    fn from(value: Date) -> Self {
        // See chrono.num_days_from_ce implementation.
        // we know this wouldn't overflow since year is limited to 1/2^13 of i32's full range.
        let mut year = value.year() - 1;
        let mut ndays = 0;
        if year < 0 {
            let excess = 1 + (-year) / 400;
            year += excess * 400;
            ndays -= excess * 146_097;
        }
        let div_100 = year / 100;
        ndays += ((year * 1461) >> 2) - div_100 + (div_100 >> 2);
        ndays += value.ordinal() as i32;

        Self(ndays)
    }
}

impl std::ops::Sub for DateLike {
    type Output = i32;

    fn sub(self, other: DateLike) -> Self::Output {
        self.0 - other.0
    }
}

impl std::ops::Sub for &DateLike {
    type Output = i32;

    fn sub(self, other: &DateLike) -> Self::Output {
        self.0 - other.0
    }
}

impl std::str::FromStr for DateLike {
    type Err = time::error::Parse;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // get only date part: yyyy-mm-dd
        // this allows to parse datetime strings
        let s = if s.len() > 10 { &s[0..10] } else { s };

        // try %Y-%m-%d
        if let Ok(d) = Date::parse(s, &format_description!("[year]-[month]-[day]")) {
            return Ok(d.into());
        }

        // try %m/%d/%Y
        Ok(Date::parse(s, &format_description!("[month]/[day]/[year]"))?.into())
    }
}

/// An error returned when the payments do not contain both negative and positive payments.
#[derive(Debug)]
pub struct InvalidPaymentsError(String);

impl fmt::Display for InvalidPaymentsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for InvalidPaymentsError {}

pub fn validate(payments: &[f64], dates: Option<&[DateLike]>) -> Result<(), InvalidPaymentsError> {
    if dates.is_some() && payments.len() != dates.unwrap_or_default().len() {
        return Err(InvalidPaymentsError(
            "the amounts and dates arrays are of different lengths".into(),
        ));
    }

    let positive = payments.iter().any(|&p| p > 0.0);
    let negative = payments.iter().any(|&p| p < 0.0);

    if positive && negative {
        Ok(())
    } else {
        Err(InvalidPaymentsError("negative and positive payments are required".into()))
    }
}
