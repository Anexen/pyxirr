use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::{create_exception, exceptions, wrap_pyfunction};

pub mod core;
use self::core::Payment;

create_exception!(pyxirr, InvalidPaymentsError, exceptions::PyException);

fn extract_iterable<'a, T>(values: &'a PyAny) -> PyResult<Vec<T>>
where
    T: FromPyObject<'a>,
{
    values.iter()?.map(|i| i.and_then(PyAny::extract::<T>)).collect()
}

fn extract_payments(dates: &PyAny, amounts: Option<&PyAny>) -> PyResult<Vec<Payment>> {
    if amounts.is_none() {
        if dates.is_instance::<PyDict>().unwrap_or(false) {
            extract_iterable::<Payment>(dates.call_method0("items")?)
        } else {
            extract_iterable::<Payment>(dates)
        }
    } else {
        Python::with_gil(|py| {
            let zipped = PyModule::import(py, "builtins")?
                .getattr("zip")?
                .call1((dates, amounts.unwrap()))?;

            extract_iterable::<Payment>(zipped)
        })
    }
}

#[pyfunction(amounts = "None", guess = "0.1")]
pub fn xirr(dates: &PyAny, amounts: Option<&PyAny>, guess: Option<f64>) -> PyResult<f64> {
    let payments = extract_payments(dates, amounts)?;

    let result = core::xirr(&payments, guess)
        .map_err(|e| exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(result)
}

#[pyfunction(amounts = "None")]
pub fn xnpv(rate: f64, dates: &PyAny, amounts: Option<&PyAny>) -> PyResult<f64> {
    let payments = extract_payments(dates, amounts)?;

    let result = core::xnpv(rate, &payments)
        .map_err(|e| exceptions::PyValueError::new_err(e.to_string()))?;

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
    use super::{core::Payment, extract_payments};
    use chrono::NaiveDate;
    use pyo3::prelude::*;
    use pyo3::types::{PyDate, PyDict, PyFloat, PyList, PyTuple};

    fn get_samples(py: Python) -> Vec<&PyAny> {
        vec![
            PyDate::new(py, 2020, 1, 1).unwrap().as_ref(),
            PyFloat::new(py, -100.123).as_ref(),
            PyDate::new(py, 2020, 2, 1).unwrap().as_ref(),
            PyFloat::new(py, 64.3).as_ref(),
        ]
    }

    fn expected_payments() -> Vec<Payment> {
        vec![
            Payment { date: NaiveDate::from_ymd(2020, 1, 1), amount: -100.123 },
            Payment { date: NaiveDate::from_ymd(2020, 2, 1), amount: 64.3 },
        ]
    }

    #[test]
    fn test_extract_from_tuples() -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let data = PyList::new(py, get_samples(py).chunks(2).map(|n| PyTuple::new(py, n)));

        let result = extract_payments(data, None)?;

        assert_eq!(result, expected_payments());
        Ok(())
    }

    #[test]
    fn test_extract_from_dict() -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let data = PyDict::new(py);

        for chunk in get_samples(py).chunks(2) {
            data.set_item(chunk[0], chunk[1])?;
        }

        let result = extract_payments(data, None)?;

        assert_eq!(result, expected_payments());
        Ok(())
    }

    #[test]
    fn test_extract_from_iterators() -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let s = get_samples(py);
        let dates = PyList::new(py, s.iter().cloned().step_by(2).collect::<Vec<&PyAny>>());
        let amounts = PyList::new(py, s.into_iter().skip(1).step_by(2).collect::<Vec<&PyAny>>());

        let result = extract_payments(dates, Some(amounts))?;

        assert_eq!(result, expected_payments());
        Ok(())
    }
}
