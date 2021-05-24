use chrono::prelude::*;
use pyo3::exceptions;
use pyo3::prelude::*;
use std::error::Error;
use std::fmt;

use pyo3::types::{PyAny, PyDate, PyDateAccess};
use std::fmt::{Display, Formatter};

const SECONDS_IN_DAY: i64 = 24 * 60 * 60;

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct DateLike(i32);

impl From<i32> for DateLike {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<&PyDate> for DateLike {
    fn from(value: &PyDate) -> Self {
        let date = NaiveDate::from_ymd(
            value.get_year(),
            value.get_month() as u32,
            value.get_day() as u32,
        );
        date.into()
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

impl std::str::FromStr for DateLike {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> chrono::ParseResult<Self> {
        Ok(s.parse::<NaiveDate>()?.into())
    }
}

impl<'s> FromPyObject<'s> for DateLike {
    fn extract(obj: &'s PyAny) -> PyResult<Self> {
        if let Ok(py_date) = obj.downcast::<PyDate>() {
            return Ok(py_date.into());
        }

        match obj.get_type().name()? {
            "datetime64" => {
                Ok(obj.call_method1("astype", ("datetime64[D]",))?.extract::<i32>()?.into())
            }
            "Timestamp" => {
                let timestamp: i64 =
                    obj.call_method0("to_pydatetime")?.call_method0("timestamp")?.extract()?;

                Ok(((timestamp / SECONDS_IN_DAY) as i32).into())
            }

            other => Err(exceptions::PyTypeError::new_err(format!(
                "Type {:?} is not understood. Expected: date",
                other
            ))),
        }
    }
}

/// An error returned when the payments do not contain both negative and positive payments.
#[derive(Debug)]
pub struct InvalidPaymentsError(String);

impl Display for InvalidPaymentsError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
