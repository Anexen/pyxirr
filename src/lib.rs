use pyo3::prelude::*;
use pyo3::{create_exception, exceptions, wrap_pyfunction};

mod conversions;
mod core;

create_exception!(pyxirr, InvalidPaymentsError, exceptions::PyException);

impl From<core::InvalidPaymentsError> for PyErr {
    fn from(value: core::InvalidPaymentsError) -> Self {
        InvalidPaymentsError::new_err(value.to_string())
    }
}

fn finite_or_none(result: f64) -> Option<f64> {
    if result.is_finite() {
        Some(result)
    } else {
        None
    }
}

#[pyfunction(amounts = "None", guess = "0.1")]
pub fn xirr(dates: &PyAny, amounts: Option<&PyAny>, guess: Option<f64>) -> PyResult<Option<f64>> {
    let (dates, amounts) = conversions::extract_payments(dates, amounts)?;
    let result = core::xirr(&dates, &amounts, guess)?;
    Ok(finite_or_none(result))
}

#[pyfunction(amounts = "None")]
pub fn xnpv(rate: f64, dates: &PyAny, amounts: Option<&PyAny>) -> PyResult<Option<f64>> {
    let (dates, amounts) = conversions::extract_payments(dates, amounts)?;
    let result = core::xnpv(rate, &dates, &amounts)?;
    Ok(finite_or_none(result))
}

#[pyfunction(guess = "0.1")]
pub fn irr(amounts: &PyAny, guess: Option<f64>) -> PyResult<Option<f64>> {
    let amounts = conversions::extract_amount_series(amounts)?;
    let result = core::irr(&amounts, guess)?;
    Ok(finite_or_none(result))
}

#[pyfunction]
pub fn npv(rate: f64, amounts: &PyAny) -> PyResult<Option<f64>> {
    let payments = conversions::extract_amount_series(amounts)?;
    let result = core::npv(rate, &payments)?;
    Ok(finite_or_none(result))
}

#[pymodule]
fn pyxirr(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(xirr))?;
    m.add_wrapped(wrap_pyfunction!(xnpv))?;
    m.add_wrapped(wrap_pyfunction!(irr))?;
    m.add_wrapped(wrap_pyfunction!(npv))?;

    m.add("InvalidPaymentsError", py.get_type::<InvalidPaymentsError>())?;

    Ok(())
}
