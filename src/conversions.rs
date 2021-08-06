use crate::core::DateLike;
use chrono::prelude::*;
use numpy::PyArray1;
use pyo3::{exceptions, prelude::*, types::*};

const SECONDS_IN_DAY: i64 = 24 * 60 * 60;

impl From<&PyDate> for DateLike {
    fn from(value: &PyDate) -> Self {
        let date =
            NaiveDate::from_ymd(value.get_year(), value.get_month() as u32, value.get_day() as u32);
        date.into()
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

fn extract_iterable<'a, T>(values: &'a PyAny) -> PyResult<Vec<T>>
where
    T: FromPyObject<'a>,
{
    values.iter()?.map(|i| i.and_then(PyAny::extract::<T>)).collect()
}

fn extract_date_series_from_numpy(series: &PyAny) -> PyResult<Vec<DateLike>> {
    Ok(series
        .call_method1("astype", ("datetime64[D]",))?
        .downcast::<PyArray1<i32>>()?
        .readonly()
        .iter()?
        .map(|&x| x.into())
        .collect())
}

pub fn extract_date_series(series: &PyAny) -> PyResult<Vec<DateLike>> {
    match series.get_type().name()? {
        "Series" => extract_date_series_from_numpy(series.getattr("values")?),
        "ndarray" => extract_date_series_from_numpy(series),
        _ => extract_iterable::<DateLike>(series),
    }
}

fn extract_amount_series_from_numpy(series: &PyAny) -> PyResult<Vec<f64>> {
    Ok(series
        .call_method1("astype", ("float64",))?
        .downcast::<PyArray1<f64>>()?
        .readonly()
        .to_vec()?)
}

fn extract_records(data: &PyAny) -> PyResult<(Vec<DateLike>, Vec<f64>)> {
    let capacity = if let Ok(capacity) = data.len() { capacity } else { 0 };

    let mut _dates: Vec<DateLike> =
        if capacity > 0 { Vec::with_capacity(capacity) } else { Vec::new() };
    let mut _amounts: Vec<f64> =
        if capacity > 0 { Vec::with_capacity(capacity) } else { Vec::new() };

    for obj in data.iter()? {
        let obj = obj?;
        // get_item() uses different ffi calls for different objects
        // PyTuple.get_item (ffi::PyTuple_GetItem) is faster than PyAny.get_item (ffi::PyObject_GetItem)
        let tup = if let Ok(py_tuple) = obj.downcast::<PyTuple>() {
            (py_tuple.get_item(0), py_tuple.get_item(1))
        } else if let Ok(py_list) = obj.downcast::<PyList>() {
            (py_list.get_item(0), py_list.get_item(1))
        } else {
            (obj.get_item(0)?, obj.get_item(1)?)
        };

        _dates.push(tup.0.extract::<DateLike>()?);
        _amounts.push(tup.1.extract::<f64>()?);
    }

    Ok((_dates, _amounts))
}

pub fn extract_amount_series(series: &PyAny) -> PyResult<Vec<f64>> {
    match series.get_type().name()? {
        "Series" => extract_amount_series_from_numpy(series.getattr("values")?),
        "ndarray" => extract_amount_series_from_numpy(series),
        _ => extract_iterable::<f64>(series),
    }
}

pub fn extract_payments(
    dates: &PyAny,
    amounts: Option<&PyAny>,
) -> PyResult<(Vec<DateLike>, Vec<f64>)> {
    if amounts.is_some() {
        return Ok((extract_date_series(dates)?, extract_amount_series(amounts.unwrap())?));
    };

    if let Ok(py_dict) = dates.downcast::<PyDict>() {
        return Ok((
            extract_iterable::<DateLike>(py_dict.keys())?,
            extract_iterable::<f64>(py_dict.values())?,
        ));
    }

    match dates.get_type().name()? {
        "DataFrame" => {
            let frame = dates;
            let columns = frame.getattr("columns")?;
            Ok((
                extract_date_series(frame.get_item(columns.get_item(0)?)?)?,
                extract_amount_series(frame.get_item(columns.get_item(1)?)?)?,
            ))
        }
        "Series"
            if dates
                .getattr("index")
                .and_then(|index| index.get_type().name())
                .unwrap_or("unknown")
                == "DatetimeIndex" =>
        {
            Ok((extract_date_series(dates.getattr("index")?)?, extract_amount_series(dates)?))
        }
        "ndarray" => {
            let array = dates;
            Ok((
                extract_date_series(array.get_item(0)?)?,
                extract_amount_series(array.get_item(1)?)?,
            ))
        }
        _ => extract_records(dates),
    }
}
