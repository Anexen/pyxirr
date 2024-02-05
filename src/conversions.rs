use numpy::{
    datetime::{units, Datetime as datetime64},
    PyArray1,
};
use pyo3::{prelude::*, types::*};

use crate::core::{DateLike, DayCount};

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

fn extract_iterable<'a, T>(values: &'a PyAny) -> PyResult<Vec<T>>
where
    T: FromPyObject<'a>,
{
    values.iter()?.map(|i| i.and_then(PyAny::extract::<T>)).collect()
}

fn extract_date_series_from_numpy(series: &PyAny) -> PyResult<Vec<DateLike>> {
    Ok(series
        .call_method1("astype", ("datetime64[D]",))?
        .downcast::<PyArray1<datetime64<units::Days>>>()?
        .readonly()
        .as_slice()?
        .iter()
        .map(|x| x.into())
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
        .extract::<&PyArray1<f64>>()?
        .readonly()
        .to_vec()?)
}

fn extract_records(data: &PyAny) -> PyResult<(Vec<DateLike>, Vec<f64>)> {
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

    for obj in data.iter()? {
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
    fn extract(obj: &'s PyAny) -> PyResult<Self> {
        extract_amount_series(obj).map(AmountArray)
    }
}

impl std::ops::Deref for AmountArray {
    type Target = [f64];

    fn deref(&self) -> &[f64] {
        self.0.as_ref()
    }
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

#[cfg(test)]
mod tests {
    use pyo3::{prelude::*, types::PyDict};
    use rstest::rstest;
    use time::{Date, Month};

    use crate::core::DateLike;

    fn get_locals<'p>(py: &'p Python) -> &'p PyDict {
        py.eval("{ 'np': __import__('numpy') }", None, None).unwrap().downcast::<PyDict>().unwrap()
    }

    #[rstest]
    #[cfg_attr(feature = "nonumpy", ignore)]
    fn test_extract_from_numpy_datetime_array() {
        Python::with_gil(|py| {
            let locals = get_locals(&py);
            let data = py
                .eval(
                    "np.array(['2007-02-01', '2009-09-30'], dtype='datetime64[D]')",
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
            let locals = get_locals(&py);
            let data = py.eval("np.datetime64('2007-02-01', '[D]')", Some(locals), None).unwrap();
            let dt: DateLike = data.extract().unwrap();
            let exp: DateLike = Date::from_calendar_date(2007, Month::February, 1).unwrap().into();

            assert_eq!(dt, exp);
        })
    }
}
