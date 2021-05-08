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
    use super::{extract_payments, core::Payment};
    use chrono::NaiveDate;
    use pyo3::prelude::*;
    use pyo3::types::PyDict;

    // fn get_samples() -> &PyAny {}

    #[test]
    fn test_extract_from_tuples() -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let date = py.import("datetime")?.getattr("date")?;
        let locals = PyDict::new(py);
        locals.set_item("date", date)?;

        let data = py.eval(
            "iter([(date(2020, 1, 1), -100.123), (date(2020, 2, 1), 64.3)])",
            None,
            Some(locals),
        )?;

        let result = extract_payments(data, None)?;

        let expected = vec![
            Payment { date: NaiveDate::from_ymd(2020, 1, 1), amount: -100.123 },
            Payment { date: NaiveDate::from_ymd(2020, 2, 1), amount: 64.3 },
        ];

        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_extract_from_iterators() -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let date = py.import("datetime")?.getattr("date")?;
        let locals = PyDict::new(py);
        locals.set_item("date", date)?;

        let data = py.eval(
            "dict([(date(2020, 1, 1), -100.123), (date(2020, 2, 1), 64.3)])",
            None,
            Some(locals),
        )?;

        let result = extract_payments(data, None)?;

        let expected = vec![
            Payment { date: NaiveDate::from_ymd(2020, 1, 1), amount: -100.123 },
            Payment { date: NaiveDate::from_ymd(2020, 2, 1), amount: 64.3 },
        ];

        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_extract_from_dict() -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let date = py.import("datetime")?.getattr("date")?;
        let locals = PyDict::new(py);
        locals.set_item("date", date)?;

        let dates = py.eval("[date(2020, 1, 1), date(2020, 2, 1)]", None, Some(locals))?;
        let amounts = py.eval("iter([-100.123, 64.3])", None, None)?;

        let result = extract_payments(dates, Some(amounts))?;

        let expected = vec![
            Payment { date: NaiveDate::from_ymd(2020, 1, 1), amount: -100.123 },
            Payment { date: NaiveDate::from_ymd(2020, 2, 1), amount: 64.3 },
        ];

        assert_eq!(result, expected);
        Ok(())
    }
}
