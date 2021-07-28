use chrono::prelude::*;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct DateLike(i32);

impl From<i32> for DateLike {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<NaiveDate> for DateLike {
    fn from(value: NaiveDate) -> Self {
        Self(value.num_days_from_ce())
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
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> chrono::ParseResult<Self> {
        Ok(s.parse::<NaiveDate>()?.into())
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
