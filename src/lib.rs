use broadcasting::Arg;
use conversions::PyDayCount;
use ndarray::ArrayD;
use pyo3::prelude::*;
use pyo3::{create_exception, exceptions, wrap_pyfunction};

mod broadcasting;
mod conversions;
mod core;

create_exception!(pyxirr, InvalidPaymentsError, exceptions::PyException);
create_exception!(pyxirr, BroadcastingError, exceptions::PyException);

impl From<core::InvalidPaymentsError> for PyErr {
    fn from(value: core::InvalidPaymentsError) -> Self {
        InvalidPaymentsError::new_err(value.to_string())
    }
}

impl From<broadcasting::BroadcastingError> for PyErr {
    fn from(value: broadcasting::BroadcastingError) -> Self {
        BroadcastingError::new_err(value.to_string())
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
#[pyfunction]
#[pyo3(signature = (dates, amounts=None, *, guess=0.1, silent=false, day_count=None))]
#[pyo3(text_signature = "(dates, amounts=None, *, guess=0.1, silent=False, day_count=None)")]
fn xirr(
    py: Python,
    dates: &PyAny,
    amounts: Option<&PyAny>,
    guess: Option<f64>,
    silent: Option<bool>,
    day_count: Option<PyDayCount>,
) -> PyResult<Option<f64>> {
    let (dates, amounts) = conversions::extract_payments(dates, amounts)?;
    let day_count = day_count.map(|x| x.try_into()).transpose()?;

    py.allow_threads(move || {
        let result = core::xirr(&dates, &amounts, guess, day_count);
        fallible_float_or_none(result, silent.unwrap_or(false))
    })
}

/// Net Present Value for a non-periodic cash flows.
#[pyfunction]
#[pyo3(signature = (rate, dates, amounts=None, *, silent=false, day_count=None))]
#[pyo3(text_signature = "(rate, dates, amounts=None, *, silent=False, day_count=None)")]
fn xnpv(
    py: Python,
    rate: f64,
    dates: &PyAny,
    amounts: Option<&PyAny>,
    silent: Option<bool>,
    day_count: Option<PyDayCount>,
) -> PyResult<Option<f64>> {
    let (dates, amounts) = conversions::extract_payments(dates, amounts)?;
    let day_count = day_count.map(|x| x.try_into()).transpose()?;

    py.allow_threads(move || {
        let result = core::xnpv(rate, &dates, &amounts, day_count);
        fallible_float_or_none(result, silent.unwrap_or(false))
    })
}

/// Internal Rate of Return
#[pyfunction]
#[pyo3(signature = (amounts, *, guess=0.1, silent=false))]
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
#[pyfunction]
#[pyo3(signature = (rate, amounts, *, start_from_zero=true))]
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
#[pyfunction]
#[pyo3(signature = (rate, nper, pmt, pv, *, pmt_at_begining=false))]
#[pyo3(text_signature = "(rate, nper, pmt, pv, *, pmt_at_begining=False)")]
fn fv(
    py: Python,
    rate: Arg,
    nper: Arg,
    pmt: Arg,
    pv: Arg,
    pmt_at_begining: Option<bool>,
) -> PyResult<PyObject> {
    use Arg::*;

    match (rate, nper, pmt, pv) {
        (Scalar(rate), Scalar(nper), Scalar(pmt), Scalar(pv)) => {
            let result = py.allow_threads(move || {
                float_or_none(core::fv(rate, nper, pmt, pv, pmt_at_begining))
            });
            return Ok(result.to_object(py));
        }
        (rate, nper, pmt, pv) => {
            let rate: ArrayD<f64> = rate.try_into()?;
            let nper: ArrayD<f64> = nper.try_into()?;
            let pmt: ArrayD<f64> = pmt.try_into()?;
            let pv: ArrayD<f64> = pv.try_into()?;

            let result = py.allow_threads(move || {
                core::periodic::fv_vec(
                    rate.view(),
                    nper.view(),
                    pmt.view(),
                    pv.view(),
                    pmt_at_begining,
                )
            })?;

            broadcasting::arrayd_to_pylist(py, result.view()).map(|x| x.into())
            // Ok(result.to_pyarray(py).to_object(py))
        }
    }
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
    signature = (start_date, cash_flow_date, end_date, cash_flow_rate, end_rate, cash_flow, *, day_count=None),
    text_signature = "(start_date, cash_flow_date, end_date, cash_flow_rate, end_rate, cash_flow, * day_count=None)"
)]
fn xfv(
    py: Python,
    start_date: core::DateLike,
    cash_flow_date: core::DateLike,
    end_date: core::DateLike,
    cash_flow_rate: f64,
    end_rate: f64,
    cash_flow: f64,
    day_count: Option<PyDayCount>,
) -> PyResult<Option<f64>> {
    let day_count = day_count.map(|x| x.try_into()).transpose()?;

    py.allow_threads(move || {
        Ok(float_or_none(core::xfv(
            &start_date,
            &cash_flow_date,
            &end_date,
            cash_flow_rate,
            end_rate,
            cash_flow,
            day_count,
        )))
    })
}

/// Net future value of a series of irregular cash flows
#[pyfunction]
#[pyo3(signature = (rate, dates, amounts=None, *, day_count=None))]
#[pyo3(text_signature = "(rate, dates, *, amounts=None, day_count=None)")]
fn xnfv(
    py: Python,
    rate: f64,
    dates: &PyAny,
    amounts: Option<&PyAny>,
    day_count: Option<PyDayCount>,
) -> PyResult<Option<f64>> {
    let (dates, amounts) = conversions::extract_payments(dates, amounts)?;
    let day_count = day_count.map(|x| x.try_into()).transpose()?;
    py.allow_threads(move || Ok(float_or_none(core::xnfv(rate, &dates, &amounts, day_count)?)))
}

/// Present Value
#[pyfunction]
#[pyo3(signature = (rate, nper, pmt, fv=0.0, *, pmt_at_begining=false))]
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
#[pyfunction]
#[pyo3(signature = (amounts, finance_rate, reinvest_rate, *, silent=false))]
#[pyo3(text_signature = "(amounts, finance_rate, reinvest_rate, *, silent=False)")]
fn mirr(
    py: Python,
    amounts: &PyAny,
    finance_rate: f64,
    reinvest_rate: f64,
    silent: Option<bool>,
) -> PyResult<Option<f64>> {
    let values = conversions::extract_amount_series(amounts)?;
    py.allow_threads(move || {
        let result = core::mirr(&values, finance_rate, reinvest_rate);
        fallible_float_or_none(result, silent.unwrap_or(false))
    })
}

/// Compute the payment against loan principal plus interest.
#[pyfunction]
#[pyo3(signature = (rate, nper, pv, fv=0.0, *, pmt_at_begining=false))]
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
#[pyfunction]
#[pyo3(signature = (rate, per, nper, pv, fv=0.0, *, pmt_at_begining=false))]
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
#[pyfunction]
#[pyo3(signature = (rate, per, nper, pv, fv=0.0, *, pmt_at_begining=false))]
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
#[pyfunction]
#[pyo3(signature = (rate, pmt, pv, fv=0.0, *, pmt_at_begining=false))]
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
#[pyfunction]
#[pyo3(signature = (nper, pmt, pv, fv=0.0, *, pmt_at_begining=false, guess=0.1))]
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

#[pyfunction]
fn year_fraction(d1: core::DateLike, d2: core::DateLike, day_count: PyDayCount) -> PyResult<f64> {
    Ok(core::year_fraction(&d1, &d2, day_count.try_into()?))
}

#[pyfunction]
fn days_between(d1: core::DateLike, d2: core::DateLike, day_count: PyDayCount) -> PyResult<i32> {
    Ok(core::days_between(&d1, &d2, day_count.try_into()?))
}

#[pymodule]
pub fn pyxirr(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<core::DayCount>()?;
    m.add_function(wrap_pyfunction!(year_fraction, m)?)?;
    m.add_function(wrap_pyfunction!(days_between, m)?)?;

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
    m.add("BroadcastingError", py.get_type::<BroadcastingError>())?;

    Ok(())
}
