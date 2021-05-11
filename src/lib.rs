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
        if dates.is_instance::<PyDict>()? {
            extract_iterable::<Payment>(dates.call_method0("items")?)
        } else {
            let values = match dates.get_type().name()? {
                "DataFrame" => dates.getattr("values")?,
                "ndarray" => dates.getattr("T")?,
                _ => dates,
            };

            extract_iterable::<Payment>(values)
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
