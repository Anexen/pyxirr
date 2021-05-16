use pyo3::prelude::*;
use pyo3::{create_exception, exceptions, wrap_pyfunction};

mod conversions;
mod core;

create_exception!(pyxirr, InvalidPaymentsError, exceptions::PyException);

#[pyfunction(amounts = "None", guess = "0.1")]
pub fn xirr(dates: &PyAny, amounts: Option<&PyAny>, guess: Option<f64>) -> PyResult<f64> {
    let payments = conversions::extract_payments(dates, amounts)?;

    let result = core::xirr(&payments, guess)
        .map_err(|e| exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(result)
}

#[pyfunction(amounts = "None")]
pub fn xnpv(rate: f64, dates: &PyAny, amounts: Option<&PyAny>) -> PyResult<f64> {
    let payments = conversions::extract_payments(dates, amounts)?;

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
