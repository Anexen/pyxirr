use chrono::{DateTime, NaiveDate, Utc};
use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDateAccess, PyIterator, PyTuple};
use pyo3::{create_exception, exceptions, wrap_pyfunction};

create_exception!(pyxirr, InvalidPaymentsError, exceptions::PyException);

fn extract_date(date: &PyDate) -> DateTime<Utc> {
    DateTime::from_utc(
        NaiveDate::from_ymd(date.get_year(), date.get_month() as u32, date.get_day() as u32)
            .and_hms(0, 0, 0),
        Utc,
    )
}

// fn extract_date_time(date: &PyDateTime) -> DateTime<Utc> {
//     DateTime::from_utc(
//         NaiveDate::from_ymd(date.get_year(), date.get_month() as u32, date.get_day() as u32)
//             .and_hms(date.get_hour() as u32, date.get_minute() as u32, date.get_second() as u32),
//         Utc,
//     )
// }

fn extract_iterable<'a, T>(py: Python<'a>, values: &PyAny) -> PyResult<Vec<T>>
where
    T: FromPyObject<'a>,
{
    PyIterator::from_object(py, values)?
        .map(|i| i.and_then(PyAny::extract::<T>))
        .collect::<PyResult<Vec<T>>>()
}

fn prepare_columnar_xirr_data(
    py: Python,
    dates: &PyAny,
    amounts: &PyAny,
) -> PyResult<(Vec<DateTime<Utc>>, Vec<f64>)> {
    let dates: Vec<DateTime<_>> =
        extract_iterable::<&PyDate>(py, dates)?.into_iter().map(extract_date).collect();

    let amounts = extract_iterable::<f64>(py, amounts)?;

    Ok((dates, amounts))
}

fn prepare_xirr_data(py: Python, data: &PyAny) -> PyResult<(Vec<DateTime<Utc>>, Vec<f64>)> {
    // data is an iterable of tuples (date, amount)

    let payments: Vec<&PyTuple> = extract_iterable::<&PyTuple>(py, data)?;

    let dates = (&payments)
        .into_iter()
        .map(|t| t.get_item(0).extract::<&PyDate>().and_then(|x| Ok(extract_date(x))))
        .collect::<PyResult<Vec<DateTime<Utc>>>>()?;

    let amounts = (&payments)
        .into_iter()
        .map(|t| t.get_item(1).extract::<f64>())
        .collect::<PyResult<Vec<f64>>>()?;

    Ok((dates.to_owned(), amounts.to_owned()))
}

#[pyfunction(amounts = "None", guess = "0.1")]
fn xirr(py: Python, dates: &PyAny, amounts: Option<&PyAny>, guess: Option<f64>) -> PyResult<f64> {
    let data = if amounts.is_none() {
        prepare_xirr_data(py, dates)?
    } else {
        prepare_columnar_xirr_data(py, dates, amounts.unwrap())?
    };

    let result = financial::xirr(&data.1, &data.0[..], guess)
        .map_err(|e| exceptions::PyValueError::new_err(e))?;

    Ok(result)
}

#[pyfunction(amounts = "None")]
fn xnpv(py: Python, rate: f64, dates: &PyAny, amounts: Option<&PyAny>) -> PyResult<f64> {
    let data = if amounts.is_none() {
        prepare_xirr_data(py, dates)?
    } else {
        prepare_columnar_xirr_data(py, dates, amounts.unwrap())?
    };

    let result = financial::xnpv(rate, &data.1, &data.0[..])
        .map_err(|e| exceptions::PyValueError::new_err(e))?;

    Ok(result)
}

#[pymodule]
fn pyxirr(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(xirr))?;
    m.add_wrapped(wrap_pyfunction!(xnpv))?;

    m.add("InvalidPaymentsError", py.get_type::<InvalidPaymentsError>())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn simple() {
        assert!(1 == 1)
    }
}
