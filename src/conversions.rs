use std::str::FromStr;

use numpy::{PyArray1, PyArrayMethods};
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
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

impl From<&Bound<'_, PyDate>> for DateLike {
    fn from(value: &Bound<'_, PyDate>) -> Self {
        let date = Date::from_calendar_date(
            value.get_year(),
            value.get_month().try_into().unwrap(),
            value.get_day(),
        )
        .unwrap();
        date.into()
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
            return Ok(py_date.into());
        }

        if let Ok(py_string) = obj.downcast::<PyString>() {
            return py_string
                .to_str()?
                .parse::<DateLike>()
                .map_err(|e| PyValueError::new_err(e.to_string()));
        }

        match obj.get_type().name()?.to_str()? {
            "datetime64" => Ok(obj
                .call_method1("astype", ("datetime64[D]",))?
                .call_method1("astype", ("int32",))?
                .extract::<DaysSinceUnixEpoch>()?
                .into()),

            "Timestamp" => Ok(obj.call_method0("to_pydatetime")?.downcast::<PyDate>()?.into()),

            other => Err(PyTypeError::new_err(format!(
                "Type {:?} is not understood. Expected: date",
                other
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
    Ok(series
        .call_method1("astype", ("datetime64[D]",))?
        .call_method1("astype", ("int32",))?
        .downcast::<PyArray1<i32>>()?
        .readonly()
        .as_slice()?
        .iter()
        .map(|&x| DateLike::from(DaysSinceUnixEpoch(x)))
        .collect())
}

pub fn extract_date_series(series: &Bound<PyAny>) -> PyResult<Vec<DateLike>> {
    match series.get_type().name()?.to_str()? {
        "Series" => extract_date_series_from_numpy(&series.getattr("values")?),
        "ndarray" => extract_date_series_from_numpy(series),
        _ => extract_iterable::<DateLike>(series),
    }
}

fn extract_amount_series_from_numpy(series: &Bound<PyAny>) -> PyResult<Vec<f64>> {
    Ok(series
        .call_method1("astype", ("float64",))?
        .extract::<numpy::PyReadonlyArray1<f64>>()?
        .to_vec()?)
}

fn extract_records(data: &Bound<PyAny>) -> PyResult<(Vec<DateLike>, Vec<f64>)> {
    let capacity = if let Ok(capacity) = data.len() {
        capacity
    } else {
        0
    };

    let mut _dates: Vec<DateLike> = if capacity > 0 {
        Vec::with_capacity(capacity)
    } else {
        Vec::new()
    };
    let mut _amounts: Vec<f64> = if capacity > 0 {
        Vec::with_capacity(capacity)
    } else {
        Vec::new()
    };

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

        _dates.push(tup.0.extract::<DateLike>()?);
        _amounts.push(tup.1.extract::<f64>()?);
    }

    Ok((_dates, _amounts))
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
    match series.get_type().name()?.to_str()? {
        "Series" => extract_amount_series_from_numpy(&series.getattr("values")?),
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

    match dates.get_type().name()?.to_str()? {
        "DataFrame" => {
            let frame = dates;
            let columns = frame.getattr("columns")?;
            Ok((
                extract_date_series(&frame.get_item(columns.get_item(0)?)?)?,
                extract_amount_series(&frame.get_item(columns.get_item(1)?)?)?,
            ))
        }
        "Series"
            if dates
                .getattr("index")
                .map(|index| {
                    index
                        .get_type()
                        .name()
                        .map(|i| i.to_str().unwrap_or_default().eq("DatetimeIndex"))
                        .unwrap_or(false)
                })
                .unwrap_or_default() =>
        {
            Ok((extract_date_series(&dates.getattr("index")?)?, extract_amount_series(dates)?))
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
