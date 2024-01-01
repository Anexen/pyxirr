use std::str::FromStr;

use crate::{DateLike, DayCount};
use pyo3::{
    create_exception,
    exceptions::{PyException, PyTypeError, PyValueError},
    prelude::*,
    types::*,
};

use numpy::datetime::{units, Datetime as datetime64};

use time::Date;

create_exception!(pyxirr, InvalidPaymentsError, PyException);
create_exception!(pyxirr, BroadcastingError, PyException);

impl From<crate::models::InvalidPaymentsError> for PyErr {
    fn from(value: crate::models::InvalidPaymentsError) -> Self {
        InvalidPaymentsError::new_err(value.to_string())
    }
}

impl From<crate::broadcasting::BroadcastingError> for PyErr {
    fn from(value: crate::broadcasting::BroadcastingError) -> Self {
        BroadcastingError::new_err(value.to_string())
    }
}

#[pymethods]
impl DayCount {
    #[staticmethod]
    pub fn of(value: &str) -> PyResult<Self> {
        DayCount::from_str(value).map_err(PyValueError::new_err)
    }

    pub fn __str__(&self) -> String {
        self.to_string()
    }
}

impl From<&PyDate> for DateLike {
    fn from(value: &PyDate) -> Self {
        let date = Date::from_calendar_date(
            value.get_year(),
            value.get_month().try_into().unwrap(),
            value.get_day(),
        )
        .unwrap();
        date.into()
    }
}

impl From<&datetime64<units::Days>> for DateLike {
    fn from(value: &datetime64<units::Days>) -> Self {
        DateLike::from_days_since_unix_epoch(i64::from(*value) as i32)
    }
}

impl<'s> FromPyObject<'s> for DateLike {
    fn extract(obj: &'s PyAny) -> PyResult<Self> {
        if let Ok(py_date) = obj.downcast::<PyDate>() {
            return Ok(py_date.into());
        }

        if let Ok(py_string) = obj.downcast::<PyString>() {
            return py_string
                .to_str()?
                .parse::<Self>()
                .map_err(|e| PyValueError::new_err(e.to_string()));
        }

        match obj.get_type().name()? {
            "datetime64" => {
                let days = obj
                    .call_method1("astype", ("datetime64[D]",))?
                    .call_method1("astype", ("int",))?
                    .extract::<i32>()?;
                Ok(DateLike::from_days_since_unix_epoch(days))
            }

            "Timestamp" => {
                let date = obj.call_method0("to_pydatetime")?.downcast::<PyDate>()?;
                Ok(date.into())
            }

            other => Err(PyTypeError::new_err(format!(
                "Type {:?} is not understood. Expected: date",
                other
            ))),
        }
    }
}
