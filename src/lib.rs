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

fn float_or_none(result: f64) -> Option<f64> {
    if result.is_nan() {
        None
    } else {
        Some(result)
    }
}

/// Internal Rate of Return for a non-periodic cash flows.
#[pyfunction(amounts = "None", guess = "0.1")]
#[pyo3(text_signature = "(dates, amounts=None, guess=0.1)")]
pub fn xirr(dates: &PyAny, amounts: Option<&PyAny>, guess: Option<f64>) -> PyResult<Option<f64>> {
    let (dates, amounts) = conversions::extract_payments(dates, amounts)?;
    let result = core::xirr(&dates, &amounts, guess)?;
    Ok(float_or_none(result))
}

/// Net Present Value for a non-periodic cash flows.
#[pyfunction(amounts = "None")]
#[pyo3(text_signature = "(rate, dates, amounts=None)")]
pub fn xnpv(rate: f64, dates: &PyAny, amounts: Option<&PyAny>) -> PyResult<Option<f64>> {
    let (dates, amounts) = conversions::extract_payments(dates, amounts)?;
    let result = core::xnpv(rate, &dates, &amounts)?;
    Ok(float_or_none(result))
}

/// Internal Rate of Return
#[pyfunction(guess = "0.1")]
#[pyo3(text_signature = "(amounts, guess=0.1)")]
pub fn irr(amounts: &PyAny, guess: Option<f64>) -> PyResult<Option<f64>> {
    let amounts = conversions::extract_amount_series(amounts)?;
    let result = core::irr(&amounts, guess)?;
    Ok(float_or_none(result))
}

/// Net Present Value.
/// NPV is calculated using the following formula:
/// sum([values[i]/(1 + rate)**i for i in range(len(values))])
/// There is a difference between numpy NPV and excel NPV.
/// By default, npv function starts from zero (numpy compatible),
/// but you can call it with `start_from_zero=False` parameter to make it Excel compatible.
#[pyfunction(start_from_zero = "true")]
#[pyo3(text_signature = "(rate, amounts, start_from_zero = True)")]
pub fn npv(rate: f64, amounts: &PyAny, start_from_zero: Option<bool>) -> PyResult<Option<f64>> {
    let payments = conversions::extract_amount_series(amounts)?;
    let result = core::npv(rate, &payments, start_from_zero);
    Ok(float_or_none(result))
}

/// Future Value.
#[pyfunction(pmt_at_begining = "false")]
#[pyo3(text_signature = "(rate, nper, pmt, pv, pmt_at_begining=False)")]
pub fn fv(rate: f64, nper: f64, pmt: f64, pv: f64, pmt_at_begining: Option<bool>) -> f64 {
    core::fv(rate, nper, pmt, pv, pmt_at_begining)
}

/// Extended Future Value.
#[pyfunction]
#[pyo3(text_signature = "(rate, nper, amounts, dates=None)")]
pub fn xfv(rate: f64, nper: f64, amounts: &PyAny, dates: Option<&PyAny>) -> PyResult<f64> {
    // uneven irregular case
    if let Some(py_dates) = dates {
        let dates = conversions::extract_date_series(py_dates)?;
        let amounts = conversions::extract_amount_series(amounts)?;
        let result = core::xfv(rate, nper, &amounts, Some(&dates))?;
        return Ok(result);
    }

    // uneven regular case
    let converted_amounts = conversions::extract_amount_series(amounts);
    if let Ok(amounts) = converted_amounts {
        return Ok(core::xfv(rate, nper, &amounts, None)?);
    }

    // ensure input is not a generator
    let _len = amounts.len()?;

    // uneven irregular case with records/dict/DataFrame input
    let (dates, amounts) = conversions::extract_payments(amounts, dates)?;
    Ok(core::xfv(rate, nper, &amounts, Some(&dates))?)
}

/// Present Value
#[pyfunction(fv = "0.0", pmt_at_begining = "false")]
#[pyo3(text_signature = "(rate, nper, pmt, fv=0, pmt_at_begining=False)")]
pub fn pv(rate: f64, nper: f64, pmt: f64, fv: Option<f64>, pmt_at_begining: Option<bool>) -> f64 {
    core::pv(rate, nper, pmt, fv, pmt_at_begining)
}

/// Modified Internal Rate of Return.
#[pyfunction]
#[pyo3(text_signature = "(amounts, finance_rate, reinvest_rate)")]
pub fn mirr(values: &PyAny, finance_rate: f64, reinvest_rate: f64) -> PyResult<Option<f64>> {
    let values = conversions::extract_amount_series(values)?;
    let result = core::mirr(&values, finance_rate, reinvest_rate);
    Ok(float_or_none(result))
}

/// Compute the payment against loan principal plus interest.
#[pyfunction(fv = "0.0", pmt_at_begining = "false")]
#[pyo3(text_signature = "(rate, nper, pv, fv=0, pmt_at_begining=False)")]
pub fn pmt(rate: f64, nper: f64, pv: f64, fv: Option<f64>, pmt_at_begining: Option<bool>) -> f64 {
    core::pmt(rate, nper, pv, fv, pmt_at_begining)
}

/// Compute the number of periodic payments.
#[pyfunction(fv = "0.0", pmt_at_begining = "false")]
#[pyo3(text_signature = "(rate, pmt, pv, fv=0, pmt_at_begining=False)")]
pub fn nper(rate: f64, pmt: f64, pv: f64, fv: Option<f64>, pmt_at_begining: Option<bool>) -> f64 {
    core::nper(rate, pmt, pv, fv, pmt_at_begining)
}

#[pymodule]
fn pyxirr(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(xirr))?;
    m.add_wrapped(wrap_pyfunction!(xnpv))?;
    m.add_wrapped(wrap_pyfunction!(irr))?;
    m.add_wrapped(wrap_pyfunction!(npv))?;
    m.add_wrapped(wrap_pyfunction!(fv))?;
    m.add_wrapped(wrap_pyfunction!(xfv))?;
    m.add_wrapped(wrap_pyfunction!(pv))?;
    m.add_wrapped(wrap_pyfunction!(pmt))?;
    m.add_wrapped(wrap_pyfunction!(nper))?;
    m.add_wrapped(wrap_pyfunction!(mirr))?;

    m.add("InvalidPaymentsError", py.get_type::<InvalidPaymentsError>())?;

    Ok(())
}
