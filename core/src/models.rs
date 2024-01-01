use std::{error::Error, fmt, str::FromStr};

use time::{macros::format_description, Date};

// time::Date::from_ordinal_date(1970, 1).unwrap().to_julian_day();
static UNIX_EPOCH_JULIAN_DAY: i32 = 2440588;

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct DateLike(Date);

impl DateLike {
    pub fn from_days_since_unix_epoch(days: i32) -> Self {
        Self(Date::from_julian_day(UNIX_EPOCH_JULIAN_DAY + days).unwrap())
    }

    pub fn from_unix_timestamp(ts: i64) -> Self {
        let days = ts.div_euclid(86400) as i32;
        Self::from_days_since_unix_epoch(days)
    }
}

impl From<DateLike> for Date {
    fn from(val: DateLike) -> Self {
        val.0
    }
}

impl From<Date> for DateLike {
    fn from(value: Date) -> Self {
        Self(value)
    }
}

impl AsRef<Date> for DateLike {
    fn as_ref(&self) -> &Date {
        &self.0
    }
}

impl FromStr for DateLike {
    type Err = time::error::Parse;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // get only date part: yyyy-mm-dd
        // this allows to parse datetime strings
        let s = if s.len() > 10 {
            &s[0..10]
        } else {
            s
        };

        // try %Y-%m-%d
        if let Ok(d) = Date::parse(s, &format_description!("[year]-[month]-[day]")) {
            return Ok(d.into());
        }

        // try %m/%d/%Y
        Ok(Date::parse(s, &format_description!("[month]/[day]/[year]"))?.into())
    }
}

/// An error returned when the payments do not contain both negative and positive payments.
#[derive(Clone, Debug)]
pub struct InvalidPaymentsError(String);

impl InvalidPaymentsError {
    pub fn new<T: fmt::Display>(message: T) -> Self {
        Self(message.to_string())
    }
}

impl fmt::Display for InvalidPaymentsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for InvalidPaymentsError {}

pub fn validate(payments: &[f64], dates: Option<&[DateLike]>) -> Result<(), InvalidPaymentsError> {
    if dates.is_some() && payments.len() != dates.unwrap_or_default().len() {
        return Err(InvalidPaymentsError::new(
            "the amounts and dates arrays are of different lengths",
        ));
    }

    let positive = payments.iter().any(|&p| p > 0.0);
    let negative = payments.iter().any(|&p| p < 0.0);

    if positive && negative {
        Ok(())
    } else {
        Err(InvalidPaymentsError::new("negative and positive payments are required"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_like_from_integer() {
        let dt = DateLike::from_unix_timestamp(1335020400);
        assert_eq!(dt.0.to_string(), "2012-04-21");

        let dt = DateLike::from_days_since_unix_epoch(15801);
        assert_eq!(dt.0.to_string(), "2013-04-06");
    }

    #[test]
    fn test_date_like_from_integer_leap_year() {
        let dt = DateLike::from_unix_timestamp(1456749295);
        assert_eq!(dt.0.to_string(), "2016-02-29");
        let dt = DateLike::from_unix_timestamp(1456835356);
        assert_eq!(dt.0.to_string(), "2016-03-01");

        let dt = DateLike::from_days_since_unix_epoch(15399);
        assert_eq!(dt.0.to_string(), "2012-02-29");
        let dt = DateLike::from_days_since_unix_epoch(15400);
        assert_eq!(dt.0.to_string(), "2012-03-01");
    }
}
