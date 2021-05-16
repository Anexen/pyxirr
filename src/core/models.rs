use chrono::prelude::*;
use pyo3::exceptions;
use pyo3::prelude::*;
use std::error::Error;
use std::fmt;

use pyo3::types::{PyAny, PyDate, PyDateAccess, PyList, PyTuple};
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
        if obj.is_instance::<PyDate>()? {
            return Ok(obj.downcast::<PyDate>()?.into());
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
                "Type {:?} is not understood",
                other
            ))),
        }
    }
}

/// A payment made or received on a particular date.
/// `amount` must be negative for payment made and positive for payment received.
#[derive(Clone, PartialEq)]
pub struct Payment {
    pub date: DateLike,
    pub amount: f64,
}

// because automatic derive FromPyObject is a bit slower + some manual tweaks
impl<'p> FromPyObject<'p> for Payment {
    fn extract(obj: &'p PyAny) -> PyResult<Self> {
        // get_item() uses different ffi calls for different objects
        // PyTuple.get_item (ffi::PyTuple_GetItem) is faster than PyAny.get_item (ffi::PyObject_GetItem)
        let tup = obj
            .downcast::<PyTuple>()
            .and_then(|tup| Ok((tup.get_item(0), tup.get_item(1))))
            .or_else(|_| -> PyResult<_> {
                // fallback to ffi::PyList_GetItem
                obj.downcast::<PyList>()
                    .and_then(|list| Ok((list.get_item(0), list.get_item(1))))
                    .or_else(|_| {
                        // fallback to ffi::PyObject_GetItem
                        Ok((obj.get_item(0)?, obj.get_item(1)?))
                    })
            })?;

        let date = tup.0.downcast::<PyDate>()?.into();
        let amount = tup.1.extract::<f64>()?;
        Ok(Self { date, amount })
    }
}

/// An error returned when the payments do not contain both negative and positive payments.
#[derive(Debug)]
pub struct InvalidPaymentsError;

impl Display for InvalidPaymentsError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        "negative and positive payments are required".fmt(f)
    }
}

impl Error for InvalidPaymentsError {}
