use chrono::{DateTime, NaiveDate, Utc};
use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDateAccess, PyDateTime, PyIterator, PyTimeAccess};
use pyo3::{create_exception, exceptions, wrap_pyfunction};

create_exception!(pyxirr, InvalidPaymentsError, exceptions::PyException);

fn extract_date(date: &PyDate) -> DateTime<Utc> {
    DateTime::from_utc(
        NaiveDate::from_ymd(date.get_year(), date.get_month() as u32, date.get_day() as u32)
            .and_hms(0, 0, 0),
        Utc,
    )
}

fn extract_date_time(date: &PyDateTime) -> DateTime<Utc> {
    DateTime::from_utc(
        NaiveDate::from_ymd(date.get_year(), date.get_month() as u32, date.get_day() as u32)
            .and_hms(date.get_hour() as u32, date.get_minute() as u32, date.get_second() as u32),
        Utc,
    )
}

fn extract_iterable<'a, T>(py: Python<'a>, values: &PyAny) -> PyResult<Vec<T>>
where
    T: FromPyObject<'a>,
{
    PyIterator::from_object(py, values)?
        .map(|i| i.and_then(PyAny::extract::<T>))
        .collect::<PyResult<Vec<T>>>()
}

#[pyfunction]
fn xirr(py: Python, dates: &PyAny, payments: &PyAny) -> PyResult<f64> {
    let dates: Vec<DateTime<_>> = match extract_iterable::<&PyDateTime>(py, dates) {
        Ok(dates) => dates.into_iter().map(extract_date_time).collect(),
        Err(_) => extract_iterable::<&PyDate>(py, dates)?.into_iter().map(extract_date).collect(),
    };

    let payments = extract_iterable::<f64>(py, payments)?;

    let result = financial::xirr(&payments, &dates[..], None)
        .map_err(|e| exceptions::PyValueError::new_err(e))?;

    Ok(result)
}

#[pymodule]
fn pyxirr(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(xirr))?;

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
