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

fn extract_amount_series(series: &PyAny) -> PyResult<Vec<f64>> {
    match series.get_type().name()? {
        "Series" => extract_amount_series_from_numpy(series.getattr("values")?),
        "ndarray" => extract_amount_series_from_numpy(series),
        _ => extract_iterable::<f64>(series),
    }
}

pub fn extract_payments(dates: &PyAny, amounts: Option<&PyAny>) -> PyResult<Vec<Payment>> {
    let dates_vec: Vec<DateLike>;
    let amounts_vec: Vec<f64>;

    if amounts.is_none() {
        if dates.is_instance::<PyDict>()? {
            return extract_iterable::<Payment>(dates.call_method0("items")?);
        }

        match dates.get_type().name()? {
            "DataFrame" => {
                let frame = dates;
                let columns = frame.getattr("columns")?;
                dates_vec = extract_date_series(frame.get_item(columns.get_item(0)?)?)?;
                amounts_vec = extract_amount_series(frame.get_item(columns.get_item(1)?)?)?;
            }
            "ndarray" => {
                let array = dates;
                dates_vec = extract_date_series(array.get_item(0)?)?;
                amounts_vec = extract_amount_series(array.get_item(1)?)?;
            }
            _ => return extract_iterable::<Payment>(dates),
        };
    } else {
        dates_vec = extract_date_series(dates)?;
        amounts_vec = extract_amount_series(amounts.unwrap())?
    }

    Ok(dates_vec
        .into_iter()
        .zip(amounts_vec)
        .map(|(date, amount)| Payment { date, amount })
        .collect())
}
