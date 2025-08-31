use std::str::FromStr;

use numpy::{PyArray1, PyArrayMethods};
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    intern,
    prelude::*,
    types::*,
};
use time::Date;

use crate::core::{DateLike, DayCount};

// time::Date::from_ordinal_date(1970, 1).unwrap().to_julian_day();
static UNIX_EPOCH_JULIAN_DAY: i32 = 2440588;

pub fn float_or_none(result: f64) -> Option<f64> {
    if result.is_nan() {
        None
    } else {
        Some(result)
    }
}

pub fn fallible_float_or_none<T>(result: Result<f64, T>, silent: bool) -> PyResult<Option<f64>>
where
    pyo3::PyErr: From<T>,
{
    match result {
        Err(e) => {
            if silent {
                Ok(None)
            } else {
                Err(e.into())
            }
        }
        Ok(v) => Ok(float_or_none(v)),
    }
}

#[derive(FromPyObject)]
pub enum PyDayCount {
    String(String),
    DayCount(DayCount),
}

impl TryInto<DayCount> for PyDayCount {
    type Error = PyErr;

    fn try_into(self) -> Result<DayCount, Self::Error> {
        match self {
            PyDayCount::String(s) => DayCount::of(&s),
            PyDayCount::DayCount(d) => Ok(d),
        }
    }
}

#[pymethods]
impl DayCount {
    #[staticmethod]
    fn of(value: &str) -> PyResult<Self> {
        DayCount::from_str(value).map_err(PyValueError::new_err)
    }

    fn __str__(&self) -> String {
        self.to_string()
    }
}

struct DaysSinceUnixEpoch(i32);

impl<'py> FromPyObject<'py> for DaysSinceUnixEpoch {
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        obj.extract::<i64>().map(|x| Self(x as i32))
    }
}

impl From<DaysSinceUnixEpoch> for DateLike {
    fn from(value: DaysSinceUnixEpoch) -> Self {
        Date::from_julian_day(UNIX_EPOCH_JULIAN_DAY + value.0).unwrap().into()
    }
}

impl From<i64> for DateLike {
    fn from(value: i64) -> Self {
        Date::from_julian_day(UNIX_EPOCH_JULIAN_DAY + (value as i32)).unwrap().into()
    }
}

impl TryFrom<&Bound<'_, PyDate>> for DateLike {
    type Error = PyErr;

    #[cfg(feature = "abi")]
    fn try_from(value: &Bound<'_, PyDate>) -> Result<Self, Self::Error> {
        let py = value.py();
        let date = Date::from_calendar_date(
            value.getattr(intern!(py, "year"))?.extract::<i32>()?,
            value.getattr(intern!(py, "month"))?.extract::<u8>()?.try_into().unwrap(),
            value.getattr(intern!(py, "day"))?.extract::<u8>()?,
        );

        Ok(date.unwrap().into())
    }

    #[cfg(not(feature = "abi"))]
    fn try_from(value: &Bound<'_, PyDate>) -> Result<Self, Self::Error> {
        let date = Date::from_calendar_date(
            value.get_year(),
            value.get_month().try_into().unwrap(),
            value.get_day(),
        );

        Ok(date.unwrap().into())
    }
}

// use numpy::datetime::{units, Datetime as datetime64};
//
// impl From<&datetime64<units::Days>> for DateLike {
//     fn from(value: &datetime64<units::Days>) -> Self {
//         let days_since_unix_epoch: i32 = Into::<i64>::into(*value) as i32;
//         let date = Date::from_julian_day(UNIX_EPOCH_JULIAN_DAY + days_since_unix_epoch).unwrap();
//
//         date.into()
//     }
// }

impl<'py> FromPyObject<'py> for DateLike {
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(py_date) = obj.downcast::<PyDate>() {
            return py_date.try_into();
        }

        if let Ok(py_string) = obj.downcast::<PyString>() {
            return py_string
                .to_cow()?
                .parse::<DateLike>()
                .map_err(|e| PyValueError::new_err(e.to_string()));
        }

        let py = obj.py();

        match obj.get_type().name()?.to_cow()?.as_ref() {
            "datetime64" => Ok(obj
                .call_method1(intern!(py, "astype"), (intern!(py, "datetime64[D]"),))?
                .call_method1(intern!(py, "astype"), (intern!(py, "int32"),))?
                .extract::<DaysSinceUnixEpoch>()?
                .into()),

            "Timestamp" => Ok(obj
                .call_method0(intern!(py, "to_pydatetime"))?
                .downcast::<PyDate>()?
                .try_into()?),

            other => Err(PyTypeError::new_err(format!(
                "Type {other:?} is not understood. Expected: date"
            ))),
        }
    }
}

fn extract_iterable<'a, T>(values: &Bound<'a, PyAny>) -> PyResult<Vec<T>>
where
    T: FromPyObject<'a>,
{
    values.try_iter()?.map(|i| i.and_then(|j| j.extract())).collect()
}

fn extract_date_series_from_numpy(series: &Bound<PyAny>) -> PyResult<Vec<DateLike>> {
    let py = series.py();
    Ok(series
        .call_method1(intern!(py, "astype"), (intern!(py, "datetime64[D]"),))?
        .call_method1(intern!(py, "astype"), (intern!(py, "int32"),))?
        .downcast::<PyArray1<i32>>()?
        .readonly()
        .as_slice()?
        .iter()
        .map(|&x| DateLike::from(DaysSinceUnixEpoch(x)))
        .collect())
}

pub fn extract_date_series(series: &Bound<PyAny>) -> PyResult<Vec<DateLike>> {
    match series.get_type().name()?.to_cow()?.as_ref() {
        "Series" => {
            let values = series.getattr(intern!(series.py(), "values"))?;
            extract_date_series_from_numpy(&values)
        }
        "ndarray" => extract_date_series_from_numpy(series),
        _ => extract_iterable::<DateLike>(series),
    }
}

fn extract_amount_series_from_numpy(series: &Bound<PyAny>) -> PyResult<Vec<f64>> {
    let py = series.py();
    Ok(series
        .call_method1(intern!(py, "astype"), (intern!(py, "float64"),))?
        .extract::<numpy::PyReadonlyArray1<f64>>()?
        .to_vec()?)
}

fn extract_records(data: &Bound<PyAny>) -> PyResult<(Vec<DateLike>, Vec<f64>)> {
    let capacity = data.len().unwrap_or(12); // pre-allocate vec
    let mut dates: Vec<DateLike> = Vec::with_capacity(capacity);
    let mut amounts: Vec<f64> = Vec::with_capacity(capacity);

    for obj in data.try_iter()? {
        let obj = obj?;
        // get_item() uses different ffi calls for different objects
        // PyTuple.get_item (ffi::PyTuple_GetItem) is faster than PyAny.get_item (ffi::PyObject_GetItem)
        let tup = if let Ok(py_tuple) = obj.downcast::<PyTuple>() {
            (py_tuple.get_item(0)?, py_tuple.get_item(1)?)
        } else if let Ok(py_list) = obj.downcast::<PyList>() {
            (py_list.get_item(0)?, py_list.get_item(1)?)
        } else {
            (obj.get_item(0)?, obj.get_item(1)?)
        };

        dates.push(tup.0.extract::<DateLike>()?);
        amounts.push(tup.1.extract::<f64>()?);
    }

    Ok((dates, amounts))
}

pub struct AmountArray(Vec<f64>);

impl AmountArray {
    pub fn into_vec(self) -> Vec<f64> {
        self.0
    }
}

impl<'s> FromPyObject<'s> for AmountArray {
    fn extract_bound(obj: &Bound<'s, PyAny>) -> PyResult<Self> {
        extract_amount_series(obj).map(AmountArray)
    }
}

impl std::ops::Deref for AmountArray {
    type Target = [f64];

    fn deref(&self) -> &[f64] {
        self.0.as_ref()
    }
}

pub fn extract_amount_series(series: &Bound<PyAny>) -> PyResult<Vec<f64>> {
    match series.get_type().name()?.to_cow()?.as_ref() {
        "Series" => {
            let values = series.getattr(intern!(series.py(), "values"))?;
            extract_amount_series_from_numpy(&values)
        }
        "ndarray" => extract_amount_series_from_numpy(series),
        _ => extract_iterable::<f64>(series),
    }
}

pub fn extract_payments(
    dates: &Bound<PyAny>,
    amounts: Option<&Bound<PyAny>>,
) -> PyResult<(Vec<DateLike>, Vec<f64>)> {
    if amounts.is_some() {
        return Ok((extract_date_series(dates)?, extract_amount_series(amounts.unwrap())?));
    };

    if let Ok(py_dict) = dates.downcast::<PyDict>() {
        return Ok((
            extract_iterable::<DateLike>(py_dict.keys().as_any())?,
            extract_iterable::<f64>(py_dict.values().as_any())?,
        ));
    }

    let py = dates.py();

    match dates.get_type().name()?.to_cow()?.as_ref() {
        "DataFrame" => {
            let frame = dates;
            let columns = frame.getattr(intern!(py, "columns"))?;
            Ok((
                extract_date_series(&frame.get_item(columns.get_item(0)?)?)?,
                extract_amount_series(&frame.get_item(columns.get_item(1)?)?)?,
            ))
        }
        "Series" => {
            let index = &dates.getattr(intern!(py, "index"))?;

            if index.get_type().name()?.ne("DatetimeIndex") {
                return Err(PyTypeError::new_err("Expected Series with DatetimeIndex"));
            }

            Ok((extract_date_series(index)?, extract_amount_series(dates)?))
        }
        "ndarray" => {
            let array = dates;
            Ok((
                extract_date_series(&array.get_item(0)?)?,
                extract_amount_series(&array.get_item(1)?)?,
            ))
        }
        _ => extract_records(dates),
    }
}

#[cfg(test)]
mod tests {
    use pyo3::{ffi::c_str, prelude::*, types::PyDict};
    use rstest::rstest;
    use time::{Date, Month};

    use crate::core::DateLike;

    fn get_locals<'p>(py: &'p Python) -> Bound<'p, PyDict> {
        py.eval(c_str!("{ 'np': __import__('numpy') }"), None, None)
            .unwrap()
            .downcast_into::<PyDict>()
            .unwrap()
    }

    #[rstest]
    #[cfg_attr(feature = "nonumpy", ignore)]
    fn test_extract_from_numpy_datetime_array() {
        Python::with_gil(|py| {
            let locals = &get_locals(&py);
            let data = py
                .eval(
                    c_str!("np.array(['2007-02-01', '2009-09-30'], dtype='datetime64[D]')"),
                    Some(locals),
                    None,
                )
                .unwrap();
            let dt: Vec<DateLike> = data.extract().unwrap();
            let exp: DateLike = Date::from_calendar_date(2007, Month::February, 1).unwrap().into();

            assert_eq!(dt[0], exp);
        })
    }

    #[rstest]
    #[cfg_attr(feature = "nonumpy", ignore)]
    fn test_extract_from_numpy_datetime() {
        Python::with_gil(|py| {
            let locals = &get_locals(&py);
            let data =
                py.eval(c_str!("np.datetime64('2007-02-01', '[D]')"), Some(locals), None).unwrap();
            let dt: DateLike = data.extract().unwrap();
            let exp: DateLike = Date::from_calendar_date(2007, Month::February, 1).unwrap().into();

            assert_eq!(dt, exp);
        })
    }
}
