use chrono::NaiveDate;
use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDateAccess, PyIterator, PyTuple};
use pyo3::{create_exception, exceptions, wrap_pyfunction};

mod xirr;
use xirr::Payment;

create_exception!(pyxirr, InvalidPaymentsError, exceptions::PyException);

// pub struct DateRange(pub NaiveDate, pub NaiveDate);

// impl Iterator for DateRange {
//     type Item = NaiveDate;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.0 <= self.1 {
//             let next = self.0 + Duration::days(1);
//             Some(std::mem::replace(&mut self.0, next))
//         } else {
//             None
//         }
//     }
// }

fn extract_date(date: &PyDate) -> NaiveDate {
    NaiveDate::from_ymd(date.get_year(), date.get_month() as u32, date.get_day() as u32)
}

fn extract_iterable<'a, T>(py: Python<'a>, values: &PyAny) -> PyResult<Vec<T>>
where
    T: FromPyObject<'a>,
{
    PyIterator::from_object(py, values)?
        .map(|i| i.and_then(PyAny::extract::<T>))
        .collect::<PyResult<Vec<T>>>()
}

fn extract_payments_from_list_of_tuples(py: Python, data: &PyAny) -> PyResult<Vec<Payment>> {
    extract_iterable::<&PyTuple>(py, data)?
        .into_iter()
        .map(|p| {
            let date = p.get_item(0).extract::<&PyDate>().and_then(|x| Ok(extract_date(x)))?;
            let amount = p.get_item(1).extract::<f64>()?;
            Ok(xirr::Payment { date, amount })
        })
        .collect::<PyResult<Vec<Payment>>>()
}

fn extract_payments_from_iterables(
    py: Python,
    dates: &PyAny,
    amounts: &PyAny,
) -> PyResult<Vec<Payment>> {
    let dates: Vec<NaiveDate> =
        extract_iterable::<&PyDate>(py, dates)?.into_iter().map(extract_date).collect();
    let amounts = extract_iterable::<f64>(py, amounts)?;

    Ok(dates.into_iter().zip(amounts).map(|(date, amount)| Payment { date, amount }).collect())
}

#[pyfunction(amounts = "None", guess = "0.1")]
pub fn xirr(
    py: Python,
    dates: &PyAny,
    amounts: Option<&PyAny>,
    guess: Option<f64>,
) -> PyResult<f64> {
    let data = if amounts.is_none() {
        extract_payments_from_list_of_tuples(py, dates)?
    } else {
        extract_payments_from_iterables(py, dates, amounts.unwrap())?
    };

    let result = xirr::xirr(&data, guess)
        .map_err(|e| exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(result)
}

#[pyfunction(amounts = "None")]
pub fn xnpv(py: Python, rate: f64, dates: &PyAny, amounts: Option<&PyAny>) -> PyResult<f64> {
    let data = if amounts.is_none() {
        extract_payments_from_list_of_tuples(py, dates)?
    } else {
        extract_payments_from_iterables(py, dates, amounts.unwrap())?
    };

    let result =
        xirr::xnpv(rate, &data).map_err(|e| exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(result)
}

#[pymodule]
fn pyxirr(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(xirr))?;
    m.add_wrapped(wrap_pyfunction!(xnpv))?;

    m.add("InvalidPaymentsError", py.get_type::<InvalidPaymentsError>())?;

    Ok(())
}
