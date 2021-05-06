use chrono::{DateTime, NaiveDate, Utc, Duration};
use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDateAccess, PyIterator, PyTuple};
use pyo3::{create_exception, exceptions, wrap_pyfunction};

mod xirr;

create_exception!(pyxirr, InvalidPaymentsError, exceptions::PyException);

pub struct DateRange(pub NaiveDate, pub NaiveDate);

impl Iterator for DateRange {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 <= self.1 {
            let next = self.0 + Duration::days(1);
            Some(std::mem::replace(&mut self.0, next))
        } else {
            None
        }
    }
}

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
pub fn xirr(
    py: Python,
    dates: &PyAny,
    amounts: Option<&PyAny>,
    guess: Option<f64>,
) -> PyResult<f64> {
    let data = if amounts.is_none() {
        prepare_xirr_data(py, dates)?
    } else {
        prepare_columnar_xirr_data(py, dates, amounts.unwrap())?
    };

    let result = financial::xirr(&data.1, &data.0[..], guess)
        .map_err(|e| exceptions::PyValueError::new_err(e))?;

    Ok(result)
}

#[pyfunction(guess = "0.1")]
pub fn faster_xirr(py: Python, data: &PyAny, guess: Option<f64>) -> PyResult<f64> {
    let data = extract_iterable::<&PyTuple>(py, data)?
        .into_iter()
        .map(|p| {
            let date = p.get_item(0).extract::<&PyDate>().and_then(|x| {
                Ok(NaiveDate::from_ymd(x.get_year(), x.get_month() as u32, x.get_day() as u32))
            })?;
            let amount = p.get_item(1).extract::<f64>()?;
            Ok(xirr::Payment { date, amount })
        })
        .collect::<PyResult<Vec<xirr::Payment>>>()?;

    let result =
        xirr::compute(&data).map_err(|e| exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(result)
}

#[pyfunction(amounts = "None")]
pub fn xnpv(py: Python, rate: f64, dates: &PyAny, amounts: Option<&PyAny>) -> PyResult<f64> {
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
    m.add_wrapped(wrap_pyfunction!(faster_xirr))?;
    m.add_wrapped(wrap_pyfunction!(xnpv))?;

    m.add("InvalidPaymentsError", py.get_type::<InvalidPaymentsError>())?;

    Ok(())
}
