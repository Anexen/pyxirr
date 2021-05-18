use crate::core::{DateLike, Payment};
use numpy::PyArray1;
use pyo3::prelude::*;
use pyo3::types::PyDict;

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

fn extract_date_series(series: &PyAny) -> PyResult<Vec<DateLike>> {
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

    if dates.is_instance::<PyDict>()? {
        return Ok((
            extract_iterable::<DateLike>(dates.call_method0("keys")?)?,
            extract_iterable::<f64>(dates.call_method0("values")?)?,
        ));
    }

    match dates.get_type().name()? {
        "DataFrame" => {
            let frame = dates;
            let columns = frame.getattr("columns")?;
            return Ok((
                extract_date_series(frame.get_item(columns.get_item(0)?)?)?,
                extract_amount_series(frame.get_item(columns.get_item(1)?)?)?,
            ));
        }
        "ndarray" => {
            let array = dates;
            return Ok((
                extract_date_series(array.get_item(0)?)?,
                extract_amount_series(array.get_item(1)?)?,
            ));
        }
        _ => {
            let data = extract_iterable::<Payment>(dates)?;
            return Ok((
                data.iter().map(|p| p.date).collect(),
                data.iter().map(|p| p.amount).collect(),
            ));
        }
    };
}

