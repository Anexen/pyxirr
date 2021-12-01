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

fn fallible_float_or_none<T>(result: Result<f64, T>, silent: bool) -> PyResult<Option<f64>>
where
    pyo3::PyErr: From<T>,
{
    match result {
        Err(e) => {
            if silent {
                Ok(None)
            } else {
                Err(e.into())
            }
        }
        Ok(v) => Ok(float_or_none(v)),
    }
}

/// Internal Rate of Return for a non-periodic cash flows.
#[pyfunction(amounts = "None", "*", guess = "0.1", silent = "false")]
#[pyo3(text_signature = "(dates, amounts, *, guess=0.1, silent=False)")]
fn xirr(
    py: Python,
    dates: &PyAny,
    amounts: Option<&PyAny>,
    guess: Option<f64>,
    silent: Option<bool>,
) -> PyResult<Option<f64>> {
    let (dates, amounts) = conversions::extract_payments(dates, amounts)?;
    py.allow_threads(move || {
        let result = core::xirr(&dates, &amounts, guess);
        fallible_float_or_none(result, silent.unwrap_or(false))
    })
}

/// Net Present Value for a non-periodic cash flows.
#[pyfunction(amounts = "None", "*", silent = "false")]
#[pyo3(text_signature = "(rate, dates, amounts, *, silent=False)")]
fn xnpv(
    py: Python,
    rate: f64,
    dates: &PyAny,
    amounts: Option<&PyAny>,
    silent: Option<bool>,
) -> PyResult<Option<f64>> {
    let (dates, amounts) = conversions::extract_payments(dates, amounts)?;
    py.allow_threads(move || {
        let result = core::xnpv(rate, &dates, &amounts);
        fallible_float_or_none(result, silent.unwrap_or(false))
    })
}

/// Internal Rate of Return
#[pyfunction("*", guess = "0.1", silent = "false")]
#[pyo3(text_signature = "(amounts, *, guess=0.1, silent=False)")]
fn irr(
    py: Python,
    amounts: &PyAny,
    guess: Option<f64>,
    silent: Option<bool>,
) -> PyResult<Option<f64>> {
    let amounts = conversions::extract_amount_series(amounts)?;
    py.allow_threads(move || {
        let result = core::irr(&amounts, guess);
        fallible_float_or_none(result, silent.unwrap_or(false))
    })
}

/// Net Present Value.
/// NPV is calculated using the following formula:
/// sum([values[i]/(1 + rate)**i for i in range(len(values))])
/// There is a difference between numpy NPV and excel NPV.
/// By default, npv function starts from zero (numpy compatible),
/// but you can call it with `start_from_zero=False` parameter to make it Excel compatible.
#[pyfunction("*", start_from_zero = "true")]
#[pyo3(text_signature = "(rate, amounts, *, start_from_zero = True)")]
fn npv(
    py: Python,
    rate: f64,
    amounts: &PyAny,
    start_from_zero: Option<bool>,
) -> PyResult<Option<f64>> {
    let payments = conversions::extract_amount_series(amounts)?;
    py.allow_threads(move || {
        let result = core::npv(rate, &payments, start_from_zero);
        Ok(float_or_none(result))
    })
}

/// Future Value.
#[pyfunction("*", pmt_at_begining = "false")]
#[pyo3(text_signature = "(rate, nper, pmt, pv, *, pmt_at_begining=False)")]
fn fv(
    py: Python,
    rate: f64,
    nper: f64,
    pmt: f64,
    pv: f64,
    pmt_at_begining: Option<bool>,
) -> Option<f64> {
    py.allow_threads(move || float_or_none(core::fv(rate, nper, pmt, pv, pmt_at_begining)))
}

/// Net Future Value.
#[pyfunction]
#[pyo3(text_signature = "(rate, nper, amounts)")]
fn nfv(py: Python, rate: f64, nper: f64, amounts: &PyAny) -> PyResult<Option<f64>> {
    let amounts = conversions::extract_amount_series(amounts)?;
    py.allow_threads(move || Ok(float_or_none(core::nfv(rate, nper, &amounts))))
}

/// Extended Future Value.
/// Future value of a cash flow between two dates.
#[pyfunction]
#[pyo3(
    text_signature = "(start_date, cash_flow_date, end_date, cash_flow_rate, end_rate, cash_flow)"
)]
fn xfv(
    py: Python,
    start_date: core::DateLike,
    cash_flow_date: core::DateLike,
    end_date: core::DateLike,
    cash_flow_rate: f64,
    end_rate: f64,
    cash_flow: f64,
) -> PyResult<Option<f64>> {
    py.allow_threads(move || {
        Ok(float_or_none(core::xfv(
            &start_date,
            &cash_flow_date,
            &end_date,
            cash_flow_rate,
            end_rate,
            cash_flow,
        )))
    })
}

/// Net future value of a series of irregular cash flows
#[pyfunction(amounts = "None")]
#[pyo3(text_signature = "(rate, dates, amounts)")]
fn xnfv(py: Python, rate: f64, dates: &PyAny, amounts: Option<&PyAny>) -> PyResult<Option<f64>> {
    let (dates, amounts) = conversions::extract_payments(dates, amounts)?;
    py.allow_threads(move || Ok(float_or_none(core::xnfv(rate, &dates, &amounts)?)))
}

/// Present Value
#[pyfunction(fv = "0.0", "*", pmt_at_begining = "false")]
#[pyo3(text_signature = "(rate, nper, pmt, fv=0, *, pmt_at_begining=False)")]
fn pv(
    py: Python,
    rate: f64,
    nper: f64,
    pmt: f64,
    fv: Option<f64>,
    pmt_at_begining: Option<bool>,
) -> Option<f64> {
    py.allow_threads(move || float_or_none(core::pv(rate, nper, pmt, fv, pmt_at_begining)))
}

/// Modified Internal Rate of Return.
#[pyfunction("*", silent = "false")]
#[pyo3(text_signature = "(amounts, finance_rate, reinvest_rate, *, silent=False)")]
fn mirr(
    py: Python,
    values: &PyAny,
    finance_rate: f64,
    reinvest_rate: f64,
    silent: Option<bool>,
) -> PyResult<Option<f64>> {
    let values = conversions::extract_amount_series(values)?;
    py.allow_threads(move || {
        let result = core::mirr(&values, finance_rate, reinvest_rate);
        fallible_float_or_none(result, silent.unwrap_or(false))
    })
}

/// Compute the payment against loan principal plus interest.
#[pyfunction(fv = "0.0", "*", pmt_at_begining = "false")]
#[pyo3(text_signature = "(rate, nper, pv, fv=0, *, pmt_at_begining=False)")]
fn pmt(
    py: Python,
    rate: f64,
    nper: f64,
    pv: f64,
    fv: Option<f64>,
    pmt_at_begining: Option<bool>,
) -> Option<f64> {
    py.allow_threads(move || float_or_none(core::pmt(rate, nper, pv, fv, pmt_at_begining)))
}

/// Compute the interest portion of a payment.
#[pyfunction(fv = "0.0", "*", pmt_at_begining = "false")]
#[pyo3(text_signature = "(rate, per, nper, pv, fv=0, *, pmt_at_begining=False)")]
fn ipmt(
    rate: f64,
    per: f64,
    nper: f64,
    pv: f64,
    fv: Option<f64>,
    pmt_at_begining: Option<bool>,
) -> Option<f64> {
    float_or_none(core::ipmt(rate, per, nper, pv, fv, pmt_at_begining))
}

/// Compute the payment against loan principal.
#[pyfunction(fv = "0.0", "*", pmt_at_begining = "false")]
#[pyo3(text_signature = "(rate, per, nper, pv, fv=0, *, pmt_at_begining=False)")]
fn ppmt(
    py: Python,
    rate: f64,
    per: f64,
    nper: f64,
    pv: f64,
    fv: Option<f64>,
    pmt_at_begining: Option<bool>,
) -> Option<f64> {
    py.allow_threads(move || float_or_none(core::ppmt(rate, per, nper, pv, fv, pmt_at_begining)))
}

/// Compute the number of periodic payments.
#[pyfunction(fv = "0.0", "*", pmt_at_begining = "false")]
#[pyo3(text_signature = "(rate, pmt, pv, fv=0, *, pmt_at_begining=False)")]
fn nper(
    py: Python,
    rate: f64,
    pmt: f64,
    pv: f64,
    fv: Option<f64>,
    pmt_at_begining: Option<bool>,
) -> Option<f64> {
    py.allow_threads(move || float_or_none(core::nper(rate, pmt, pv, fv, pmt_at_begining)))
}

/// Compute the number of periodic payments.
#[pyfunction(fv = "0.0", "*", pmt_at_begining = "false", guess = "0.1")]
#[pyo3(text_signature = "(nper, pmt, pv, fv=0, *, pmt_at_begining=False, guess=0.1)")]
fn rate(
    py: Python,
    nper: f64,
    pmt: f64,
    pv: f64,
    fv: Option<f64>,
    pmt_at_begining: Option<bool>,
    guess: Option<f64>,
) -> Option<f64> {
    py.allow_threads(move || float_or_none(core::rate(nper, pmt, pv, fv, pmt_at_begining, guess)))
}

#[pymodule]
pub fn pyxirr(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(pmt, m)?)?;
    m.add_function(wrap_pyfunction!(ipmt, m)?)?;
    m.add_function(wrap_pyfunction!(ppmt, m)?)?;
    m.add_function(wrap_pyfunction!(nper, m)?)?;
    m.add_function(wrap_pyfunction!(rate, m)?)?;
    m.add_function(wrap_pyfunction!(fv, m)?)?;
    m.add_function(wrap_pyfunction!(nfv, m)?)?;
    m.add_function(wrap_pyfunction!(xfv, m)?)?;
    m.add_function(wrap_pyfunction!(xnfv, m)?)?;
    m.add_function(wrap_pyfunction!(pv, m)?)?;
    m.add_function(wrap_pyfunction!(npv, m)?)?;
    m.add_function(wrap_pyfunction!(xnpv, m)?)?;
    m.add_function(wrap_pyfunction!(irr, m)?)?;
    m.add_function(wrap_pyfunction!(mirr, m)?)?;
    m.add_function(wrap_pyfunction!(xirr, m)?)?;

    m.add("InvalidPaymentsError", py.get_type::<InvalidPaymentsError>())?;

    Ok(())
}
